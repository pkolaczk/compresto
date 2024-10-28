mod codec;
mod discard;

use crate::discard::Discard;
use anyhow::bail;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use clap::{Args, Parser, Subcommand, ValueEnum};
use codec::{brotli, lzma};
use std::cmp::min;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Cursor, Error, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::{Duration, Instant};
use human_bytes::human_bytes;
use serde::Serialize;

#[derive(Parser)]
struct Config {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compress a file
    Compress(CompressionCfg),
    /// Decompress a file
    Decompress(DecompressionCfg),
    /// Benchmark compression+decompression of a single file
    Benchmark(CompressionCfg),
    /// Run multiple benchmarks
    BenchmarkMany(BenchmarkManyCfg),
}

#[derive(Args, Clone)]
struct InputCfg {
    /// Input file path
    #[arg()]
    path: PathBuf,

    /// Path to a dictionary file
    #[arg(long, short = 'd')]
    dict: Option<PathBuf>,

    /// Length of the dictionary prefix to use
    #[arg(long, default_value = "16384")]
    dict_len: u64,
}

#[derive(Args)]
struct CompressionCfg {
    #[clap(flatten)]
    input: InputCfg,

    /// Compression algorithm
    #[arg(long, short = 'a', default_value = "zstd")]
    algorithm: Algorithm,

    /// Compression level
    #[arg(long, short = 'c', default_value = "1", allow_hyphen_values = true)]
    compression: i32,

    /// Size of a file chunk in bytes. Each chunk is compressed independently.
    #[arg(long, short = 'b', default_value = "16384")]
    chunk_size: usize,
}

#[derive(Args)]
struct DecompressionCfg {
    #[clap(flatten)]
    input: InputCfg,

    /// Compression algorithm. If not given, determined automatically from the file extension.
    #[clap(long, short = 'a')]
    algorithm: Option<Algorithm>,
}

#[derive(Args)]
struct BenchmarkManyCfg {
    #[clap(flatten)]
    input: InputCfg,

    /// List of algorithms to benchmark
    #[arg(long, short = 'a', value_delimiter = ',', default_value = "lz4,lzav,snappy,zstd,brotli", num_args = 1..)]
    algorithms: Vec<Algorithm>,

    /// Size of a file chunk in bytes. Each chunk is compressed independently.
    #[arg(long, short = 'b', default_value = "16384")]
    chunk_size: usize,

    /// Save benchmark results to a CSV file
    #[arg(long, short)]
    report: Option<PathBuf>
}

#[derive(ValueEnum, Copy, Clone, Serialize)]
enum Algorithm {
    Copy,
    Lz4,
    Zstd,
    Brotli,
    Snappy,
    Lzma,
    Lzav,
}

impl Algorithm {
    fn extension(&self) -> &str {
        match self {
            Algorithm::Copy => "bak",
            Algorithm::Zstd => "zstd",
            Algorithm::Lz4 => "lz4",
            Algorithm::Brotli => "br",
            Algorithm::Snappy => "sz",
            Algorithm::Lzma => "xz",
            Algorithm::Lzav => "lzav",
        }
    }

    fn from_file_name(path: &Path) -> Option<Algorithm> {
        match path.extension().and_then(OsStr::to_str) {
            Some("bak") => Some(Self::Copy),
            Some("zstd") => Some(Self::Zstd),
            Some("lz4") => Some(Self::Lz4),
            Some("br") => Some(Self::Brotli),
            Some("sz") => Some(Self::Snappy),
            Some("xz") => Some(Self::Lzma),
            Some("lzav") => Some(Self::Lzav),
            _ => None,
        }
    }

    fn get_compression_levels(&self) -> Vec<i32> {
        match self {
            Algorithm::Copy => vec![0],
            Algorithm::Zstd => Vec::from_iter((-7..=-1).chain(1..=12)),
            Algorithm::Lz4 => Vec::from_iter((-9..=-1).chain(1..=9)),
            Algorithm::Brotli => Vec::from_iter(1..=8),
            Algorithm::Snappy => vec![0],
            Algorithm::Lzma => vec![0],
            Algorithm::Lzav => vec![0, 1],
        }
    }
}

struct Measurement {
    input_len: u64,
    output_len: u64,
    elapsed: Duration,
}

impl Measurement {
    fn compression_ratio(&self) -> f64 {
        self.output_len as f64 / self.input_len as f64
    }

    fn input_throughtput(&self) -> f64 {
        self.input_len as f64 / self.elapsed.as_secs_f64()
    }

    fn output_throughtput(&self) -> f64 {
        self.output_len as f64 / self.elapsed.as_secs_f64()
    }

    fn format_compression(&self) -> String {
        format!(
            "{} => {} ({:.1} %)",
            self.input_len,
            self.output_len,
            self.compression_ratio() * 100.0
        )
    }
}

#[derive(Serialize)]
struct BenchmarkResult {
    algorithm: Algorithm,
    level: i32,
    uncompressed_len: u64,
    compressed_len: u64,
    ratio: f64,
    inv_ratio: f64,
    compression_speed_mpbs: f64,
    decompression_speed_mpbs: f64,
}

impl BenchmarkResult {
    fn new(cfg: CompressionCfg, compression: Measurement, decompression: Measurement) -> Self {
        Self {
            algorithm: cfg.algorithm,
            level: cfg.compression,
            uncompressed_len: compression.input_len,
            compressed_len: compression.output_len,
            ratio: (compression.compression_ratio() * 1000.0).round() / 1000.0,
            inv_ratio: (1.0 / compression.compression_ratio() * 1000.0).round() / 1000.0, 
            compression_speed_mpbs: (compression.input_throughtput() / 100_000.0).round() / 10.0,
            decompression_speed_mpbs: (decompression.output_throughtput() / 100_000.0).round() / 10.0,
        }
    }
}

impl Display for BenchmarkResult {
    
    
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:10} lev. {:3}:    {:8} => {:8} ({:5.1}%, {:4.2}x),    compr.: {:6.1} MB/s, decompr.: {:6.1} MB/s",
            self.algorithm
                .to_possible_value()
                .unwrap_or_default()
                .get_name(),
            self.level,
            human_bytes(self.uncompressed_len as f64),
            human_bytes(self.compressed_len as f64),
            self.ratio * 100.0,
            1.0 / self.ratio,
            self.compression_speed_mpbs,
            self.decompression_speed_mpbs
        )
    }
}

fn main() {
    let cmd = Config::parse();
    if let Err(e) = run(cmd) {
        eprintln!("error: {}", e);
        exit(1);
    }
}

fn run(cmd: Config) -> anyhow::Result<()> {
    match cmd.command {
        Command::Decompress(cfg) => run_decompress_cmd(cfg),
        Command::Compress(cfg) => run_compress_cmd(cfg),
        Command::Benchmark(cfg) => run_benchmark_cmd(cfg).map(|_| ()),
        Command::BenchmarkMany(cfg) => run_benchmark_many_cmd(cfg),
    }
}

fn run_decompress_cmd(cfg: DecompressionCfg) -> anyhow::Result<()> {
    let Some(algorithm) = cfg
        .algorithm
        .or_else(|| Algorithm::from_file_name(&cfg.input.path))
    else {
        bail!("Cannot determine compression algorithm from the extension. Please use -a/--algorithm option.");
    };

    let dict = dictionary(&cfg.input)?;
    let mut decoder = decoder(algorithm, dict.as_ref())?;
    let input = open_input(&cfg.input)?;
    let output = open_output(&cfg.input.path, algorithm, false)?;
    let result = decompress(input, output, decoder.as_mut())?;
    eprintln!(
        "{}, {:.1} MB/s",
        result.format_compression(),
        result.output_throughtput() / 1_000_000.0
    );
    Ok(())
}

fn run_compress_cmd(cfg: CompressionCfg) -> anyhow::Result<()> {
    let dict = dictionary(&cfg.input)?;
    let mut encoder = encoder(cfg.algorithm, cfg.compression, dict.as_ref())?;
    let input = open_input(&cfg.input)?;
    let output = open_output(&cfg.input.path, cfg.algorithm, true)?;
    let result = compress(input, output, cfg.chunk_size, encoder.as_mut())?;
    eprintln!(
        "{}, {:.1} MB/s",
        result.format_compression(),
        result.input_throughtput() / 1_000_000.0
    );
    Ok(())
}

fn run_benchmark_cmd(cfg: CompressionCfg) -> anyhow::Result<BenchmarkResult> {
    let dict = dictionary(&cfg.input)?;
    let mut encoder = encoder(cfg.algorithm, cfg.compression, dict.as_ref())?;
    let mut decoder = decoder(cfg.algorithm, dict.as_ref())?;

    let mut input = open_input(&cfg.input)?;
    let mut buffered_input = Vec::new();
    input.read_to_end(&mut buffered_input)?;
    let input_len = buffered_input.len();
    let mut input = Cursor::new(buffered_input);

    let mut output = Cursor::new(Vec::<u8>::with_capacity(input_len));

    let c_perf = compress(&mut input, &mut output, cfg.chunk_size, encoder.as_mut())?;
    output.rewind()?;
    let d_perf = decompress(output, Discard::default(), decoder.as_mut())?;
    let result = BenchmarkResult::new(cfg, c_perf, d_perf);
    println!("{}", result);
    Ok(result)
}

fn run_benchmark_many_cmd(cfg: BenchmarkManyCfg) -> anyhow::Result<()> {
    let mut results = Vec::new();
    
    for algorithm in cfg.algorithms {
        for level in algorithm.get_compression_levels() {
            let run_cfg = CompressionCfg {
                input: cfg.input.clone(),
                algorithm,
                compression: level,
                chunk_size: cfg.chunk_size,
            };
            results.push(run_benchmark_cmd(run_cfg)?);
        }
    }
    
    if let Some(path) = cfg.report {
        let mut writer = csv::Writer::from_path(path)?;
        for result in results {
            writer.serialize(&result)?;   
        }
        writer.flush()?;
    }
    
    Ok(())
}

fn open_input(config: &InputCfg) -> Result<File, Error> {
    File::open(&config.path).map_err(|e| {
        Error::new(
            e.kind(),
            format!("Could not open file {}: {}", config.path.display(), e),
        )
    })
}

fn open_output(input_path: &Path, algorithm: Algorithm, compress: bool) -> Result<File, Error> {
    let extension_suffix = if compress { algorithm.extension() } else { "" };

    let new_extension = match input_path.extension() {
        None => extension_suffix.to_owned(),
        Some(ext) => format!("{}.{}", ext.to_string_lossy(), extension_suffix),
    };
    let output_path = input_path.with_extension(new_extension);
    let output = File::create(&output_path).map_err(|e| {
        Error::new(
            e.kind(),
            format!("Could not create file {}: {}", output_path.display(), e),
        )
    })?;
    Ok(output)
}

fn encoder(
    algorithm: Algorithm,
    compression: i32,
    dict: Option<&Vec<u8>>,
) -> anyhow::Result<Box<dyn codec::Encoder>> {
    Ok(match (algorithm, dict) {
        (Algorithm::Copy, _) => Box::new(codec::copy::Copy),
        (Algorithm::Lz4, _) => Box::new(codec::lz4::Lz4Compressor::new(compression)),
        (Algorithm::Zstd, None) => Box::new(zstd::bulk::Compressor::new(compression)?),
        (Algorithm::Zstd, Some(dict)) => {
            Box::new(zstd::bulk::Compressor::with_dictionary(compression, dict)?)
        }
        (Algorithm::Brotli, None) => Box::new(brotli::BrotliCompressor(compression)),
        (Algorithm::Brotli, Some(dict)) => {
            Box::new(brotli::BrotliDictCompressor::new(compression as u32, dict))
        }
        (Algorithm::Snappy, _) => Box::new(snap::raw::Encoder::new()),
        (Algorithm::Lzma, _) => Box::new(lzma::LzmaCompressor(compression as u32)),
        (Algorithm::Lzav, _) => Box::new(codec::lzav::LzavCompressor::new(compression as u32)),
    })
}

fn decoder(
    algorithm: Algorithm,
    dict: Option<&Vec<u8>>,
) -> anyhow::Result<Box<dyn codec::Decoder>> {
    Ok(match (algorithm, dict) {
        (Algorithm::Copy, _) => Box::new(codec::copy::Copy),
        (Algorithm::Lz4, _) => Box::new(codec::lz4::Lz4Decompressor),
        (Algorithm::Zstd, None) => Box::new(zstd::bulk::Decompressor::new()?),
        (Algorithm::Zstd, Some(dict)) => Box::new(zstd::bulk::Decompressor::with_dictionary(dict)?),
        (Algorithm::Brotli, None) => Box::new(brotli::BrotliDecompressor),
        (Algorithm::Brotli, Some(dict)) => Box::new(brotli::BrotliDictDecompressor::new(dict)),
        (Algorithm::Snappy, _) => Box::new(snap::raw::Decoder::new()),
        (Algorithm::Lzma, _) => Box::new(lzma::LzmaDecompressor),
        (Algorithm::Lzav, _) => Box::new(codec::lzav::LzavDecompressor),
    })
}

fn dictionary(input_cfg: &InputCfg) -> io::Result<Option<Vec<u8>>> {
    match input_cfg.dict.as_ref() {
        None => Ok(None),
        Some(p) => Ok(Some(load_dictionary(p, input_cfg.dict_len).map_err(
            |e| {
                Error::new(
                    e.kind(),
                    format!("Failed to load dictionary {}: {}", p.display(), e),
                )
            },
        )?)),
    }
}

fn load_dictionary(path: &Path, len: u64) -> io::Result<Vec<u8>> {
    let mut dict_input = File::open(path)?;
    let to_read = min(len, dict_input.metadata()?.len()) as usize;
    let mut data = vec![0_u8; to_read];
    let mut ptr = &mut data[0..];
    while !ptr.is_empty() {
        let count = dict_input.read(ptr)?;
        ptr = &mut ptr[count..];
    }
    Ok(data)
}

fn compress<R: Read + Seek, W: Write + Seek>(
    input: R,
    output: W,
    chunk_size: usize,
    encoder: &mut dyn codec::Encoder,
) -> anyhow::Result<Measurement> {
    let input = BufReader::with_capacity(chunk_size, input);
    let mut tmp_buf = vec![0; encoder.compressed_len_bound(chunk_size)];

    measure(input, output, |input, output| {
        while !input.fill_buf()?.is_empty() {
            let input_chunk = input.buffer();
            let uncompressed_len = input_chunk.len();
            let compressed_len = encoder.compress(input_chunk, &mut tmp_buf)?;
            output.write_u32::<LittleEndian>(uncompressed_len.try_into().unwrap())?;
            output.write_u32::<LittleEndian>(compressed_len.try_into().unwrap())?;
            output.write_all(&tmp_buf[0..compressed_len])?;
            input.consume(uncompressed_len);
        }
        output.flush()?;
        Ok(())
    })
}

fn decompress<R: Read + Seek, W: Write + Seek>(
    input: R,
    output: W,
    decoder: &mut dyn codec::Decoder,
) -> anyhow::Result<Measurement> {
    let input = BufReader::with_capacity(256 * 1024 * 1024, input);
    let mut src = Vec::new();
    let mut dest = Vec::new();

    measure(input, output, |input, output| {
        while !input.fill_buf()?.is_empty() {
            let uncompressed_len = input.read_u32::<LittleEndian>()?.try_into().unwrap();
            let frame_len = input.read_u32::<LittleEndian>()?.try_into().unwrap();
            dest.resize(uncompressed_len, 0);
            if input.buffer().len() >= frame_len {
                let src = &input.buffer()[0..frame_len];
                let count = decoder.decompress(src, &mut dest)?;
                assert_eq!(count, uncompressed_len);
                input.consume(frame_len);
            } else {
                src.resize(frame_len, 0);
                input.read_exact(&mut src)?;
                let count = decoder.decompress(&src, &mut dest)?;
                assert_eq!(count, uncompressed_len);
            }
            output.write_all(&dest)?;
        }
        output.flush()?;
        Ok(())
    })
}

/// Measure performance of compression or decompression
fn measure<I: Seek, O: Seek, T>(
    mut input: I,
    mut output: O,
    mut process: impl FnMut(&mut I, &mut O) -> anyhow::Result<T>,
) -> anyhow::Result<Measurement> {
    let start_time = Instant::now();
    process(&mut input, &mut output)?;
    let end_time = Instant::now();
    let input_pos = input.stream_position()?;
    let output_pos = output.stream_position()?;

    Ok(Measurement {
        input_len: input_pos,
        output_len: output_pos,
        elapsed: end_time - start_time,
    })
}
