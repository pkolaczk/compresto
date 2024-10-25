use std::io;
use std::io::{ErrorKind, Seek, SeekFrom, Write};

#[derive(Default)]
pub struct Discard {
    pos: u64,
    max_pos: u64,
}

impl Write for Discard {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.pos += buf.len() as u64;
        self.max_pos = self.max_pos.max(self.pos);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Seek for Discard {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let (base_pos, offset) = match pos {
            SeekFrom::Start(count) => (count, 0),
            SeekFrom::End(count) => (self.max_pos, count),
            SeekFrom::Current(count) => (self.pos, count),
        };
        self.pos = match base_pos.checked_add_signed(offset) {
            Some(ok) => ok,
            None => {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "invalid seek to a negative or overflowing position",
                ))
            }
        };
        self.max_pos = self.max_pos.max(self.pos);
        Ok(self.pos)
    }
}
