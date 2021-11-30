use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct ReadWrite(Sender<u8>, Receiver<u8>);

pub fn read_write() -> (ReadWrite, ReadWrite) {
    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();

    (ReadWrite(tx1, rx2), ReadWrite(tx2, rx1))
}

impl Read for ReadWrite {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for (i, byte) in buf.iter_mut().enumerate() {
            match self.1.recv() {
                Ok(i) => {
                    *byte = i;
                },
                Err(_) => {
                    return Ok(i);
                },
            }
        }

        Ok(buf.len())
    }
}

impl Write for ReadWrite {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for (i, byte) in buf.iter().enumerate() {
            match self.0.send(*byte) {
                Ok(()) => {},
                Err(_) => {
                    return Ok(i);
                },
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
