use crate::codec::{Decoder, Encoder};
use std::ffi::c_int;
use std::io::ErrorKind;

enum CompressionLevel { Default, Hi }

pub struct LzavCompressor(CompressionLevel);

impl LzavCompressor {
    pub(crate) fn new(compression_level: u32) -> Self {
        match compression_level {
            0 => LzavCompressor(CompressionLevel::Default),
            1 => LzavCompressor(CompressionLevel::Hi),
            _ => unimplemented!(),
        }
    }
}

pub struct LzavDecompressor;

impl Encoder for LzavCompressor {
    fn compressed_len_bound(&mut self, uncompressed_len: usize) -> usize {
        unsafe {
            match self.0 {
                CompressionLevel::Default => lzav::compress_bound(uncompressed_len as c_int) as usize,
                CompressionLevel::Hi => lzav::compress_bound_hi(uncompressed_len as c_int) as usize,
            }
        }
    }

    fn compress(&mut self, src: &[u8], dest: &mut [u8]) -> std::io::Result<usize> {
        unsafe {
            let count = match self.0 {
                CompressionLevel::Default => lzav::compress_default(
                    src.as_ptr() as *const _,
                    dest.as_mut_ptr() as *mut _,
                    src.len() as c_int,
                    dest.len() as c_int,
                ),
                CompressionLevel::Hi => lzav::compress_hi(
                    src.as_ptr() as *const _,
                    dest.as_mut_ptr() as *mut _,
                    src.len() as c_int,
                    dest.len() as c_int,
                )
            };
            if count == 0 {
                Err(std::io::Error::new(
                    ErrorKind::Other,
                    "lzav compress failed",
                ))
            } else {
                Ok(count as usize)
            }
        }
    }
}

impl Decoder for LzavDecompressor {
    fn decompress(&mut self, src: &[u8], dest: &mut [u8]) -> std::io::Result<usize> {
        unsafe {
            let count = lzav::decompress(
                src.as_ptr() as *const _,
                dest.as_mut_ptr() as *mut _,
                src.len() as c_int,
                dest.len() as c_int,
            );
            if count == 0 {
                Err(std::io::Error::new(
                    ErrorKind::Other,
                    "lzav decompress failed",
                ))
            } else {
                Ok(count as usize)
            }
        }
    }
}
