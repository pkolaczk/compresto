mod decoder;
mod encoder;

use crate::decoder::{CopyFrom, Decoder};
use crate::encoder::{CopyTo, Encoder};
use anyhow::bail;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::cmp::min;
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Error, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::{Duration, Instant};
use zstd::dict::{DecoderDictionary, EncoderDictionary};

#[derive(Parser)]
struct Config {
    #[command(subcommand)]
    command: Command,

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

#[derive(Subcommand)]
enum Command {
    /// Compress a file
    Compress(CompressionCfg),
    /// Decompress a file
    Decompress {
        /// Compression algorithm. If not given, determined automatically from the file extension.
        #[clap(long, short = 'a')]
        algorithm: Option<Algorithm>,
    },

    /// Benchmark compression+decompression of a single run
    Benchmark(CompressionCfg),
}

#[derive(Args)]
struct CompressionCfg {
    /// Compression algorithm
    #[clap(long, short = 'a', default_value = "zstd")]
    algorithm: Algorithm,

    /// Compression level
    #[clap(long, short = 'c', default_value = "1", allow_hyphen_values = true)]
    compression: i32,

    /// Size of a file chunk in bytes. Each chunk is compressed independently.
    #[clap(long, short = 'b', default_value = "16384")]
    chunk_size: u64,
}

#[derive(ValueEnum, Copy, Clone)]
enum Algorithm {
    Copy,
    Lz4,
    Zstd,
}

impl Algorithm {
    fn extension(&self) -> &str {
        match self {
            Algorithm::Copy => "bak",
            Algorithm::Zstd => "zstd",
            Algorithm::Lz4 => "lz4",
        }
    }

    fn from_file_name(path: &Path) -> Option<Algorithm> {
        match path.extension().and_then(OsStr::to_str) {
            Some("bak") => Some(Self::Copy),
            Some("zstd") => Some(Self::Zstd),
            Some("lz4") => Some(Self::Lz4),
            _ => None,
        }
    }
}

enum Encoding {
    Lz4 {
        level: u32,
    },
    Zstd {
        level: i32,
        dict: Option<EncoderDictionary<'static>>,
    },
    Copy,
}

enum Decoding {
    Lz4,
    Zstd {
        dict: Option<DecoderDictionary<'static>>,
    },
    Copy,
}

impl Encoding {
    fn new_encoder<'a, W: Write + 'a>(&self, output: W) -> io::Result<Box<dyn Encoder + 'a>> {
        Ok(match self {
            Encoding::Lz4 { level } => Box::new(
                lz4::EncoderBuilder::new()
                    .favor_dec_speed(true)
                    .level(*level)
                    .build(output)?,
            ),
            Encoding::Zstd { level, dict } => match &dict {
                Some(dict) => Box::new(zstd::Encoder::with_prepared_dictionary(output, &dict)?),
                None => Box::new(zstd::Encoder::new(output, *level)?),
            },
            Encoding::Copy => Box::new(CopyTo::new(output)),
        })
    }
}

impl Decoding {
    fn new_decoder<'a, R: BufRead + 'a>(&self, input: R) -> io::Result<Box<dyn Decoder + 'a>> {
        Ok(match self {
            Self::Lz4 { .. } => Box::new(lz4::Decoder::new(input)?),
            Self::Zstd { dict, .. } => match &dict {
                Some(dict) => Box::new(zstd::Decoder::with_prepared_dictionary(input, &dict)?),
                None => Box::new(zstd::Decoder::new(input)?),
            },
            Self::Copy => Box::new(CopyFrom::new(input)),
        })
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

fn main() {
    let cmd = Config::parse();
    if let Err(e) = run(cmd) {
        eprintln!("error: {}", e);
        exit(1);
    }
}

fn run(cmd: Config) -> anyhow::Result<()> {
    let mut input = File::open(&cmd.path).map_err(|e| {
        Error::new(
            e.kind(),
            format!("Could not open file {}: {}", cmd.path.display(), e),
        )
    })?;

    let dict = match &cmd.dict {
        None => None,
        Some(path) => Some(load_dictionary(&path, cmd.dict_len).map_err(|e| {
            Error::new(
                e.kind(),
                format!("Failed to load dictionary {}: {}", path.display(), e),
            )
        })?),
    };

    match &cmd.command {
        Command::Decompress { algorithm } => {
            let Some(algorithm) = algorithm.and_then(|_| Algorithm::from_file_name(&cmd.path))
            else {
                bail!("Cannot determine compression algorithm from the extension. Please use -a/--algorithm option.");
            };
            let decoding = decoding(dict.as_ref(), algorithm);
            let output = open_output(&cmd)?;
            let result = decompress(input, output, &decoding)?;
            eprintln!(
                "{}, {:.1} MB/s",
                result.format_compression(),
                result.output_throughtput() / 1_000_000.0
            );
        }
        Command::Compress(CompressionCfg {
            algorithm,
            compression,
            chunk_size,
        }) => {
            let encoding = encoding(dict.as_ref(), *algorithm, *compression);
            let output = open_output(&cmd)?;
            let result = compress(input, output, *chunk_size, &encoding)?;
            eprintln!(
                "{}, {:.1} MB/s",
                result.format_compression(),
                result.input_throughtput() / 1_000_000.0
            );
        }
        Command::Benchmark(CompressionCfg {
            algorithm,
            compression,
            chunk_size,
        }) => {
            let encoding = encoding(dict.as_ref(), *algorithm, *compression);
            let decoding = decoding(dict.as_ref(), *algorithm);
            let mut buffered_input = Vec::new();
            input.read_to_end(&mut buffered_input)?;
            let mut input = Cursor::new(buffered_input);
            let mut output = Cursor::new(Vec::<u8>::new());
            let c_perf = compress(&mut input, &mut output, *chunk_size, &encoding)?;
            output.rewind()?;
            let d_perf = decompress(output, Cursor::new(Vec::<u8>::new()), &decoding)?;
            println!(
                "{}, compression: {:.1} MB/s, decompression: {:.1} MB/s",
                c_perf.format_compression(),
                c_perf.input_throughtput() / 1_000_000.0,
                d_perf.output_throughtput() / 1_000_000.0
            );
        }
    }
    Ok(())
}

fn open_output(cmd: &Config) -> Result<File, Error> {
    let extension_suffix = match &cmd.command {
        Command::Compress(CompressionCfg { algorithm, .. }) => algorithm.extension(),
        Command::Decompress { .. } => "",
        Command::Benchmark(_) => "",
    };

    let new_extension = match cmd.path.extension() {
        None => extension_suffix.to_owned(),
        Some(ext) => format!("{}.{}", ext.to_string_lossy(), extension_suffix),
    };
    let output_path = cmd.path.clone().with_extension(new_extension);

    let output = File::create(&output_path).map_err(|e| {
        Error::new(
            e.kind(),
            format!("Could not create file {}: {}", output_path.display(), e),
        )
    })?;
    Ok(output)
}

fn decoding(dict: Option<&Vec<u8>>, algorithm: Algorithm) -> Decoding {
    let decoding = match algorithm {
        Algorithm::Copy => Decoding::Copy,
        Algorithm::Lz4 => Decoding::Lz4,
        Algorithm::Zstd => Decoding::Zstd {
            dict: dict.map(|d| DecoderDictionary::copy(d)),
        },
    };
    decoding
}

fn encoding(dict: Option<&Vec<u8>>, algorithm: Algorithm, compression: i32) -> Encoding {
    let encoding = match algorithm {
        Algorithm::Copy => Encoding::Copy,
        Algorithm::Lz4 => Encoding::Lz4 {
            level: compression.try_into().unwrap_or_default(),
        },
        Algorithm::Zstd => Encoding::Zstd {
            level: compression,
            dict: dict.map(|d| EncoderDictionary::copy(d, compression)),
        },
    };
    encoding
}

fn load_dictionary(path: &Path, len: u64) -> io::Result<Vec<u8>> {
    let mut dict_input = File::open(&path)?;
    let to_read = min(len, dict_input.metadata()?.len()) as usize;
    let mut data = vec![0_u8; to_read];
    let mut ptr = &mut data[0..];
    while ptr.len() > 0 {
        let count = dict_input.read(ptr)?;
        ptr = &mut ptr[count..];
    }
    Ok(data)
}

fn compress<R: Read + Seek, W: Write + Seek>(
    input: R,
    output: W,
    chunk_size: u64,
    compression: &Encoding,
) -> io::Result<Measurement> {
    let input = BufReader::new(input);
    let output = BufWriter::new(output);
    measure(input, output, |mut i, mut o| {
        Ok(while compress_chunk(&mut i, &mut o, chunk_size, compression)? == chunk_size {})
    })
}

fn compress_chunk<R: BufRead, W: Write + Seek>(
    mut input: R,
    output: W,
    chunk_size: u64,
    compression: &Encoding,
) -> io::Result<u64> {
    let mut encoder = compression.new_encoder(output)?;
    let mut remaining = chunk_size;
    let mut total_read = 0;
    let mut buf: [u8; 16384] = [0; 16384];
    while remaining > 0 {
        let to_read = min(remaining, buf.len() as u64) as usize;
        let buf = &mut buf[0..to_read];
        let count = input.read(buf)?;
        if count == 0 {
            break;
        }
        remaining -= count as u64;
        total_read += count as u64;
        let slice = &buf[..count];
        encoder.write_all(slice)?;
    }
    encoder.finish()?;
    Ok(total_read)
}

fn decompress<R: Read + Seek, W: Write + Seek>(
    input: R,
    output: W,
    decoding: &Decoding,
) -> io::Result<Measurement> {
    let input = BufReader::new(input);
    let output = BufWriter::new(output);
    measure(input, output, |mut i, mut o| {
        Ok(while decompress_chunk(&mut i, &mut o, decoding)? > 0 {})
    })
}

fn decompress_chunk<R: BufRead, W: Write + Seek>(
    mut input: R,
    mut output: W,
    decoding: &Decoding,
) -> io::Result<u64> {
    if input.fill_buf()?.is_empty() {
        return Ok(0);
    }
    let mut decoder = decoding.new_decoder(input)?;
    let mut total_read = 0;
    let mut buf: [u8; 16384] = [0; 16384];
    loop {
        let count = decoder.read(&mut buf)?;
        if count == 0 {
            break;
        }
        total_read += count as u64;
        output.write_all(&buf[..count])?;
    }
    Ok(total_read)
}

/// Measure performance of compression or decompression
fn measure<I: Seek, O: Seek, T>(
    mut input: I,
    mut output: O,
    process: impl Fn(&mut I, &mut O) -> io::Result<T>,
) -> io::Result<Measurement> {
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
