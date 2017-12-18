use std::io::{Read, Write, Result};
use std::marker::PhantomData;

pub struct Channel<R: Read, W: Write> {
    buffer: Box<[u8]>,
    begin: usize,
    end: usize,
    capacity: usize,
    ph_read: PhantomData<R>,
    ph_write: PhantomData<W>,
    pub src_closed: bool,
    pub dest_closed: bool,
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
            src_closed: false,
            dest_closed: false,
        }
    }

    pub fn recv_bytes(&mut self, src: &mut R) -> Result<ChannelResult> {
        if self.src_closed {
            return Ok(ChannelResult::ReadClosed);
        }

        let limit = if self.begin <= self.end {
            self.capacity
        } else {
            self.begin
        };

        if limit == self.end {
            return Ok(ChannelResult::Success(0));
        }

        let read = src.read(&mut self.buffer[self.end..limit])?;

        //println!("----RECV----begin:{}, end:{}", self.begin, self.end);
//        let string = String::from_utf8_lossy(&self.buffer[self.end..(self.end + read)]);
//        print!("{}", string);

        if read == 0 {
            //println!("CLOSING");
            self.src_closed = true;
            return Ok(ChannelResult::ReadClosed);
        }

        self.end += read;

        //println!("----RECVEND----begin:{}, end:{}", self.begin, self.end);

        Ok(ChannelResult::Success(read))
    }

    pub fn send_bytes(&mut self, dest: &mut W) -> Result<ChannelResult> {
        if self.dest_closed {
            return Ok(ChannelResult::WriteClosed);
        }

        let limit = if self.begin <= self.end {
            self.end
        } else {
            self.capacity
        };

        if limit == self.begin {
            return Ok(ChannelResult::Success(0));
        }

        let write = dest.write(&mut self.buffer[self.begin..limit])?;

        //println!("----SEND----begin:{}, end:{}", self.begin, self.end);
//        let string = String::from_utf8_lossy(&self.buffer[self.begin..(self.begin + write)]);
//        print!("{}", string);

        self.begin += write;

        if self.end == self.capacity && self.begin > 0 {
            self.end = 0
        }

        if self.begin == self.capacity {
            self.begin = 0
        }

        //println!("----SENDEND----begin:{}, end:{}", self.begin, self.end);

        Ok(ChannelResult::Success(write))
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

pub enum ChannelResult {
    Success(usize),
    ReadClosed,
    WriteClosed
}

