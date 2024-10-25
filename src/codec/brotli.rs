use crate::codec::{Decoder, Encoder};
use brotlic_sys::{
    BrotliDecoderAttachDictionary, BrotliDecoderCreateInstance, BrotliDecoderDecompress,
    BrotliDecoderDecompressStream, BrotliDecoderDestroyInstance, BrotliDecoderHasMoreOutput,
    BrotliEncoderAttachPreparedDictionary, BrotliEncoderCompress, BrotliEncoderCompressStream,
    BrotliEncoderCreateInstance, BrotliEncoderDestroyInstance,
    BrotliEncoderDestroyPreparedDictionary, BrotliEncoderHasMoreOutput,
    BrotliEncoderMaxCompressedSize, BrotliEncoderMode_BROTLI_MODE_GENERIC,
    BrotliEncoderOperation_BROTLI_OPERATION_FINISH, BrotliEncoderParameter_BROTLI_PARAM_MODE,
    BrotliEncoderParameter_BROTLI_PARAM_QUALITY, BrotliEncoderPrepareDictionary,
    BrotliEncoderPreparedDictionary, BrotliEncoderSetParameter,
    BrotliSharedDictionaryType_BROTLI_SHARED_DICTIONARY_RAW, BROTLI_DEFAULT_WINDOW,
    BROTLI_MAX_QUALITY,
};
use std::ffi::c_int;
use std::io::ErrorKind;
use std::{io, ptr};

pub struct BrotliCompressor(pub i32);
pub struct BrotliDecompressor;

impl Encoder for BrotliCompressor {
    fn compressed_len_bound(&mut self, uncompressed_len: usize) -> usize {
        unsafe { BrotliEncoderMaxCompressedSize(uncompressed_len) }
    }

    fn compress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize> {
        let input_ptr = src.as_ptr();
        let input_len = src.len();
        let output_ptr = dest.as_mut_ptr();
        let mut output_len = dest.len();

        let result = unsafe {
            BrotliEncoderCompress(
                self.0,
                BROTLI_DEFAULT_WINDOW as c_int,
                BrotliEncoderMode_BROTLI_MODE_GENERIC,
                input_len,
                input_ptr,
                &mut output_len,
                output_ptr,
            )
        };
        if result != 0 {
            Ok(output_len)
        } else {
            Err(io::Error::new(ErrorKind::Other, "Failed to compress"))
        }
    }
}

impl Decoder for BrotliDecompressor {
    fn decompress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize> {
        let input_ptr = src.as_ptr();
        let input_len = src.len();
        let output_ptr = dest.as_mut_ptr();
        let mut output_len = dest.len();

        let result =
            unsafe { BrotliDecoderDecompress(input_len, input_ptr, &mut output_len, output_ptr) };
        if result != 0 {
            Ok(output_len)
        } else {
            Err(io::Error::new(ErrorKind::Other, "Failed to decompress"))
        }
    }
}

pub struct BrotliDictCompressor {
    quality: u32,
    dict: *mut BrotliEncoderPreparedDictionary,
}

impl BrotliDictCompressor {
    pub fn new(quality: u32, dict: &[u8]) -> Self {
        let dict_data = dict.to_vec();

        unsafe {
            let dict = BrotliEncoderPrepareDictionary(
                BrotliSharedDictionaryType_BROTLI_SHARED_DICTIONARY_RAW,
                dict_data.len(),
                dict_data.as_ptr(),
                BROTLI_MAX_QUALITY as c_int,
                None,
                None,
                ptr::null_mut(),
            );

            if dict.is_null() {
                panic!("BrotliEncoderPrepareDictionary returned NULL")
            }

            BrotliDictCompressor { quality, dict }
        }
    }
}

impl Drop for BrotliDictCompressor {
    fn drop(&mut self) {
        unsafe {
            BrotliEncoderDestroyPreparedDictionary(self.dict);
        }
    }
}

impl Encoder for BrotliDictCompressor {
    fn compressed_len_bound(&mut self, uncompressed_len: usize) -> usize {
        unsafe { BrotliEncoderMaxCompressedSize(uncompressed_len) }
    }

    fn compress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let instance = BrotliEncoderCreateInstance(None, None, ptr::null_mut());
            if instance.is_null() {
                panic!(
                    "BrotliEncoderCreateInstance returned NULL: failed to allocate or initialize"
                );
            }

            if BrotliEncoderSetParameter(
                instance,
                BrotliEncoderParameter_BROTLI_PARAM_MODE,
                BrotliEncoderMode_BROTLI_MODE_GENERIC as u32,
            ) == 0
            {
                panic!("Failed to set compression mode");
            };
            if BrotliEncoderSetParameter(
                instance,
                BrotliEncoderParameter_BROTLI_PARAM_QUALITY,
                self.quality,
            ) == 0
            {
                panic!("Failed to set compression quality");
            }

            if BrotliEncoderAttachPreparedDictionary(instance, self.dict) == 0 {
                panic!("BrotliEncoderAttachPreparedDictionary failed")
            };

            let mut input_ptr = src.as_ptr();
            let mut input_len = src.len();
            let mut output_ptr = dest.as_mut_ptr();
            let mut output_len = dest.len();
            let mut total_out = 0;

            loop {
                if BrotliEncoderCompressStream(
                    instance,
                    BrotliEncoderOperation_BROTLI_OPERATION_FINISH,
                    &mut input_len,
                    &mut input_ptr,
                    &mut output_len,
                    &mut output_ptr,
                    &mut total_out,
                ) == 0
                {
                    return Err(io::Error::new(ErrorKind::Other, "Failed to compress"));
                };

                if BrotliEncoderHasMoreOutput(instance) == 0 {
                    break;
                }
            }

            BrotliEncoderDestroyInstance(instance);
            Ok(total_out)
        }
    }
}

pub struct BrotliDictDecompressor {
    dict: Vec<u8>,
}

impl BrotliDictDecompressor {
    pub fn new(dict: &[u8]) -> Self {
        BrotliDictDecompressor {
            dict: dict.to_vec(),
        }
    }
}

impl Decoder for BrotliDictDecompressor {
    fn decompress(&mut self, src: &[u8], dest: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let instance = BrotliDecoderCreateInstance(None, None, ptr::null_mut());
            if instance.is_null() {
                panic!(
                    "BrotliDecoderCreateInstance returned NULL: failed to allocate or initialize"
                );
            }

            if BrotliDecoderAttachDictionary(
                instance,
                BrotliSharedDictionaryType_BROTLI_SHARED_DICTIONARY_RAW,
                self.dict.len(),
                self.dict.as_ptr(),
            ) == 0
            {
                panic!("BrotliDecoderAttachDictionary failed")
            };

            let mut input_ptr = src.as_ptr();
            let mut input_len = src.len();
            let mut output_ptr = dest.as_mut_ptr();
            let mut output_len = dest.len();
            let mut total_out = 0;

            loop {
                if BrotliDecoderDecompressStream(
                    instance,
                    &mut input_len,
                    &mut input_ptr,
                    &mut output_len,
                    &mut output_ptr,
                    &mut total_out,
                ) == 0
                {
                    return Err(io::Error::new(ErrorKind::Other, "Failed to decompress"));
                };

                if BrotliDecoderHasMoreOutput(instance) == 0 {
                    break;
                }
            }

            BrotliDecoderDestroyInstance(instance);
            Ok(total_out)
        }
    }
}
