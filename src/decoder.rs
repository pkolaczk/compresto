use std::io;
use std::io::BufRead;

pub trait Decoder {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize>;
}

impl<R: BufRead> Decoder for zstd::Decoder<'_, R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        io::Read::read(self, bytes)
    }
}

impl<R: BufRead> Decoder for lz4::Decoder<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        io::Read::read(self, bytes)
    }
}

impl<R: BufRead> Decoder for brotlic::DecompressorReader<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        io::Read::read(self, bytes)
    }
}

pub struct CopyFrom<R: BufRead>(R);

impl<R: BufRead> CopyFrom<R> {
    pub fn new(input: R) -> CopyFrom<R> {
        CopyFrom(input)
    }
}

impl<R: BufRead> Decoder for CopyFrom<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.0.read(bytes)
    }
}
