use std::io;
use zstd::zstd_safe;
use crate::codec::{Decoder, Encoder};

impl Encoder for zstd::bulk::Compressor<'_> {
    fn compressed_len_bound(&mut self, src_len: usize) -> usize {
        zstd_safe::compress_bound(src_len)
    }

    fn compress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize> {
        self.compress_to_buffer(src, dest)
    }
}

impl Decoder for zstd::bulk::Decompressor<'_> {
    fn decompress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize> {
        self.decompress_to_buffer(src, dest)
    }
}