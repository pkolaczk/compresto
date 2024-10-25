use std::io;
use crate::codec::{Decoder, Encoder};

pub struct Copy;

impl Encoder for Copy {
    fn compressed_len_bound(&mut self, uncompressed_len: usize) -> usize {
        uncompressed_len
    }

    fn compress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize> {
        dest[0..src.len()].copy_from_slice(src);
        Ok(src.len())
    }
}

impl Decoder for Copy {
    fn decompress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize> {
        dest[0..src.len()].copy_from_slice(src);
        Ok(src.len())
    }
}
