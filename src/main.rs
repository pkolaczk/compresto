mod decoder;
mod encoder;

use crate::decoder::{CopyFrom, Decoder};
use crate::encoder::{CopyTo, Encoder};
use clap::{Parser, ValueEnum};
use std::cmp::min;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Error, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::Instant;
use zstd::dict::{DecoderDictionary, EncoderDictionary};

#[derive(Parser)]
struct Command {
    /// Input file path
    #[clap()]
    path: PathBuf,

    /// Compression algorithm
    #[clap(long, short = 'a', default_value = "zstd")]
    algorithm: Algorithm,

    /// Compression level
    #[clap(long, short = 'c', default_value = "1", allow_hyphen_values = true)]
    compression: i32,

    /// Size of a file chunk in bytes. Each chunk is compressed independently.
    #[clap(long, short = 'b', default_value = "16384")]
    chunk_size: u64,

    /// Path to a dictionary file
    #[clap(long, short = 'd')]
    dict: Option<PathBuf>,

    /// Length of the dictionary prefix to use
    #[clap(long, default_value = "16384")]
    dict_len: u64,

    /// Decompress instead of compressing
    #[clap(short = 'x', required = false)]
    extract: bool,
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

struct Summary {
    input_len: u64,
    output_len: u64,
    throughput_bps: u64,
}

fn main() {
    let cmd = Command::parse();
    match run(cmd) {
        Ok(summary) => {
            eprintln!(
                "{} => {} ({:.1}%) {:.1} MB/s",
                summary.input_len,
                summary.output_len,
                (summary.output_len as f32) / (summary.input_len as f32) * 100.0,
                summary.throughput_bps as f64 / 1000000.0
            )
        }
        Err(e) => {
            eprintln!("error: {}", e);
            exit(1);
        }
    }
}

fn run(cmd: Command) -> io::Result<Summary> {
    let input = File::open(&cmd.path).map_err(|e| {
        Error::new(
            e.kind(),
            format!("Could not open file {}: {}", cmd.path.display(), e),
        )
    })?;

    let extension_suffix = if cmd.extract {
        "x"
    } else {
        cmd.algorithm.extension()
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

    let dict = match cmd.dict {
        None => None,
        Some(path) => Some(load_dictionary(&path, cmd.dict_len).map_err(|e| {
            Error::new(
                e.kind(),
                format!("Failed to load dictionary {}: {}", path.display(), e),
            )
        })?),
    };

    let start_time = Instant::now();
    let (orig_size, out_size) = if cmd.extract {
        let decoding = match cmd.algorithm {
            Algorithm::Copy => Decoding::Copy,
            Algorithm::Lz4 => Decoding::Lz4,
            Algorithm::Zstd => Decoding::Zstd {
                dict: dict.map(|d| DecoderDictionary::copy(&d)),
            },
        };
        decompress(input, output, &decoding)?
    } else {
        let encoding = match cmd.algorithm {
            Algorithm::Copy => Encoding::Copy,
            Algorithm::Lz4 => Encoding::Lz4 {
                level: cmd.compression.try_into().unwrap_or_default(),
            },
            Algorithm::Zstd => Encoding::Zstd {
                level: cmd.compression,
                dict: dict.map(|d| EncoderDictionary::copy(&d, cmd.compression)),
            },
        };
        compress(input, output, cmd.chunk_size, &encoding)?
    };

    let end_time = Instant::now();
    let elapsed = end_time - start_time;
    let throughput_bps = (orig_size as f64 / elapsed.as_secs_f64()) as u64;
    Ok(Summary {
        input_len: orig_size,
        output_len: out_size,
        throughput_bps,
    })
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

fn compress(
    input: File,
    output: File,
    chunk_size: u64,
    compression: &Encoding,
) -> io::Result<(u64, u64)> {
    let mut input = BufReader::new(input);
    let mut output = BufWriter::new(output);
    while compress_chunk(&mut input, &mut output, chunk_size, compression)? == chunk_size {}

    let orig_size = input.stream_position()?;
    let compressed_size = output.stream_position()?;
    Ok((orig_size, compressed_size))
}

fn compress_chunk(
    input: &mut BufReader<File>,
    output: &mut BufWriter<File>,
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

fn decompress(input: File, output: File, decoding: &Decoding) -> io::Result<(u64, u64)> {
    let mut input = BufReader::new(input);
    let mut output = BufWriter::new(output);
    while decompress_chunk(&mut input, &mut output, decoding)? > 0 {}

    let orig_size = input.stream_position()?;
    let decompressed_size = output.stream_position()?;
    Ok((orig_size, decompressed_size))
}

fn decompress_chunk(
    input: &mut BufReader<File>,
    output: &mut BufWriter<File>,
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
