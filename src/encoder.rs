use std::io;
use std::io::Write;

pub trait Encoder {
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()>;
    fn finish(self: Box<Self>) -> io::Result<()>;
}

impl<W: Write> Encoder for zstd::Encoder<'_, W> {
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        Write::write_all(self, bytes)
    }

    fn finish(self: Box<Self>) -> io::Result<()> {
        zstd::Encoder::finish(*self)?;
        Ok(())
    }
}

impl<W: Write> Encoder for lz4::Encoder<W> {
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        Write::write_all(self, bytes)
    }

    fn finish(self: Box<Self>) -> io::Result<()> {
        let (_, result) = lz4::Encoder::finish(*self);
        result
    }
}

impl<W: Write> Encoder for brotlic::CompressorWriter<W> {
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        Write::write_all(self, bytes)
    }

    fn finish(mut self: Box<Self>) -> io::Result<()> {
        self.flush()?;
        Ok(())
    }
}

impl<W: Write> Encoder for snap::write::FrameEncoder<W> {
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        Write::write_all(self, bytes)
    }

    fn finish(mut self: Box<Self>) -> io::Result<()> {
        self.flush()?;
        Ok(())
    }
}

pub struct CopyTo<W: Write>(W);

impl<W: Write> CopyTo<W> {
    pub fn new(output: W) -> CopyTo<W> {
        CopyTo(output)
    }
}

impl<W: Write> Encoder for CopyTo<W> {
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.0.write_all(bytes)
    }

    fn finish(mut self: Box<Self>) -> io::Result<()> {
        self.0.flush()
    }
}
