use std::io;
use std::io::{Cursor, Read, Seek, Write};
use lzma_sys::lzma_stream_buffer_bound;
use crate::codec::{Decoder, Encoder};

pub struct LzmaCompressor(pub u32);
pub struct LzmaDecompressor;

impl Encoder for LzmaCompressor {
    fn compressed_len_bound(&mut self, uncompressed_len: usize) -> usize {
        unsafe { lzma_stream_buffer_bound(uncompressed_len) }
    }

    fn compress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize> {
        let w = Cursor::new(dest);
        let mut encoder = xz2::write::XzEncoder::new(w, self.0);
        encoder.write_all(src)?;
        let mut w = encoder.finish()?;
        Ok(w.stream_position()? as usize)
    }
}

impl Decoder for LzmaDecompressor {
    fn decompress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize> {
        let r = Cursor::new(src);
        let mut decoder = xz2::read::XzDecoder::new(r);
        decoder.read_exact(dest)?;
        Ok(dest.len())
    }
}
