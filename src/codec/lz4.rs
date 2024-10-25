use std::io;
use lz4::block::CompressionMode;
use crate::codec::{Decoder, Encoder};

pub struct Lz4Compressor(pub CompressionMode);

impl Lz4Compressor {
    pub(crate) fn new(compression: i32) -> Lz4Compressor {
        match compression {
            ..0 => Lz4Compressor(CompressionMode::FAST(-compression)),
            0 => Lz4Compressor(CompressionMode::DEFAULT),
            _ => Lz4Compressor(CompressionMode::HIGHCOMPRESSION(compression)),
        }
    }
}

pub struct Lz4Decompressor;

impl Encoder for Lz4Compressor {
    fn compressed_len_bound(&mut self, uncompressed_len: usize) -> usize {
        lz4::block::compress_bound(uncompressed_len).unwrap()
    }

    fn compress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize> {
        lz4::block::compress_to_buffer(src, Some(self.0), false, dest)
    }
}

impl Decoder for Lz4Decompressor {
    fn decompress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize> {
        lz4::block::decompress_to_buffer(src, Some(dest.len() as i32), dest)
    }
}

