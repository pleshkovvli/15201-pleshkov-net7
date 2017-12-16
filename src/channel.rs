use std::io::{Read, Write, Result};
use std::marker::PhantomData;
pub struct Channel<R: Read, W: Write> {
    buffer: Box<[u8]>,
    begin: usize,
    end: usize,
    capacity: usize,
    ph_read: PhantomData<R>,
    ph_write: PhantomData<W>,

}

impl<'a, R: Read, W: Write> Channel<R, W> {
    pub fn new(buf_size: usize) -> Channel<R, W> {
        Channel {
            buffer: Box::from(vec![0; buf_size]),
            begin: 0,
            end: 0,
            capacity: buf_size,
            ph_read: PhantomData,
            ph_write: PhantomData,
        }
    }

    pub fn recv_bytes(&mut self, src: &mut R) -> Result<usize> {
        let limit = if self.begin <= self.end {
            self.capacity
        } else {
            self.begin
        };

        if limit == self.end {
            return Ok(0);
        }

        let read =  src.read(&mut self.buffer[self.end..limit])?;
        self.end += read;

        Ok(read)
    }

    pub fn send_bytes(&mut self, dest: &mut W) -> Result<usize> {
        let limit = if self.begin <= self.end {
            self.end
        } else {
            self.capacity
        };

        if limit == self.begin {
            return Ok(0);
        }

        let write = dest.write(&mut self.buffer[self.begin..limit])?;
        self.begin += write;

        if self.end == self.capacity && self.begin > 0 {
            self.end = 0
        }

        if self.begin == self.capacity {
            self.begin = 0
        }

        Ok(write)
    }

    pub fn free_space(&self) -> usize {
        if self.begin <= self.end {
            self.capacity - self.end + self.begin
        } else {
            self.begin - self.end - 1
        }
    }

    pub fn bytes_available(&self) -> usize {
        if self.begin <= self.end {
            self.end - self.begin
        } else {
            self.capacity + self.end - self.begin
        }
    }
}

