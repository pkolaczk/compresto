use crate::codec::{Decoder, Encoder};
use std::io;
use std::io::ErrorKind;

impl Encoder for snap::raw::Encoder {
    fn compressed_len_bound(&mut self, uncompressed_len: usize) -> usize {
        snap::raw::max_compress_len(uncompressed_len)
    }

    fn compress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize> {
        snap::raw::Encoder::compress(self, src, dest)
            .map_err(|e| io::Error::new(ErrorKind::Other, e))
    }
}

impl Decoder for snap::raw::Decoder {
    fn decompress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize> {
        snap::raw::Decoder::decompress(self, src, dest)
            .map_err(|e| io::Error::new(ErrorKind::Other, e))
    }
}
