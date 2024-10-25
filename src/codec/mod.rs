use std::io;

pub mod brotli;
pub mod copy;
pub mod lz4;
pub mod lzav;
pub mod lzma;
pub mod snappy;
pub mod zstd;

pub trait Encoder {
    fn compressed_len_bound(&mut self, uncompressed_len: usize) -> usize;
    fn compress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize>;
}

pub trait Decoder {
    fn decompress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize>;
}
