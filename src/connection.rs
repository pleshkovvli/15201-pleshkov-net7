extern crate mio;

use channel::{Channel, ChannelResult};

use std::net::Shutdown;

use mio::net::TcpStream;
use mio::Token;
use mio::Ready;
use mio::Event;
use std::io::Result;

const BUFFER_SIZE: usize = 8192;

type TcpChannel = Channel<TcpStream, TcpStream>;

pub struct Connection {
    client: TokenStream,
    server: TokenStream,
    from_client: TcpChannel,
    from_server: TcpChannel,
    closing: bool,
}

pub struct TokenStream {
    pub token: Token,
    pub stream: TcpStream,
}

pub struct EventChannels<'a> {
    from_event: &'a mut TcpChannel,
    from_other: &'a mut TcpChannel,
    event_stream: &'a mut TcpStream,
    other_stream: &'a mut TcpStream,
    closing: &'a bool,
    event_token: Token,
    other_token: Token,
}

impl<'a> EventChannels<'a> {
    fn read_from_event(&mut self) -> Result<ChannelResult> {
        println!("READING FROM {}", self.event_stream.peer_addr()?);

        match self.from_event.recv_bytes(self.event_stream)? {
            ChannelResult::ReadClosed => {
                self.closing = &true;
                Ok(ChannelResult::ReadClosed)
            }
            ChannelResult::Success(i) => {
                Ok(ChannelResult::Success(i))
            }
            ChannelResult::WriteClosed => {
                Ok(ChannelResult::WriteClosed)
            }
        }
    }

    fn write_to_event(&mut self) -> Result<ChannelResult> {
        println!("WRITING ON {}", self.event_stream.peer_addr()?);
        self.from_other.send_bytes(self.event_stream)
    }

    fn check_shutdown(from: &'a mut TcpChannel, dest_stream: &'a mut TcpStream) -> Result<bool> {
        if from.src_closed && !from.dest_closed {
            if from.bytes_available() == 0 {
                dest_stream.shutdown(Shutdown::Write)?;
                from.dest_closed = true;
                return Ok(true);
            }
        }

        return Ok(false);
    }

    fn check_shutdowns(&mut self) -> Result<bool> {
        if *self.closing {
            Self::check_shutdown(self.from_event, self.other_stream)?;
            Self::check_shutdown(self.from_other, self.event_stream)?;
        }

        Ok(false)
    }

    fn all_closed(&self) -> bool {
        return self.from_event.src_closed
            && self.from_event.dest_closed
            && self.from_other.src_closed
            && self.from_other.dest_closed;
    }
}

pub struct TokenReady<'a> {
    pub token: Token,
    pub stream: &'a TcpStream,
    pub ready: Ready,
}

impl Connection {
    pub fn new(client: TokenStream, server: TokenStream) -> Connection {
        let from_client: TcpChannel = Channel::new(BUFFER_SIZE);
        let from_server: TcpChannel = Channel::new(BUFFER_SIZE);

        Connection { client, server, from_client, from_server, closing: false }
    }

    fn get_stream_ready(channels: &EventChannels) -> Ready {
        let mut ready = Ready::empty();
        if channels.from_event.free_space() > 0 {
            ready.insert(Ready::readable());
        }
        if channels.from_other.bytes_available() > 0 {
            ready.insert(Ready::writable());
        }

        ready
    }

    fn get_stream_ready_reverse(channels: &EventChannels) -> Ready {
        let mut ready = Ready::empty();
        if channels.from_other.free_space() > 0 {
            ready.insert(Ready::readable());
        }
        if channels.from_event.bytes_available() > 0 {
            ready.insert(Ready::writable());
        }

        ready
    }

    pub fn tokens(&self) -> (Token, Token) {
        (self.client.token, self.server.token)
    }

    pub fn handle_event(&mut self, token: Token, event: Event) -> Result<ConnectionResult> {
        let ready = event.readiness();

        let mut event_channels: EventChannels = self.channels_on_token(token)
            .expect("Unknown token on handle event");

        if ready.is_readable() {
            event_channels.read_from_event()?;
        }

        if ready.is_writable() {
            event_channels.write_to_event()?;
        }

        event_channels.check_shutdowns()?;

        if event_channels.all_closed() {
            return Ok(ConnectionResult::Close)
        }

        Ok(ConnectionResult::Continue(
            TokenReady {
                token: event_channels.event_token,
                stream: event_channels.event_stream,
                ready: Self::get_stream_ready(&event_channels),
            },
            TokenReady {
                token: event_channels.other_token,
                stream: event_channels.other_stream,
                ready: Self::get_stream_ready_reverse(&event_channels),
            },
        ))
    }

//    fn check_shutdown(from: &mut TcpChannel, dest_stream: &mut TcpStream) -> Result<bool> {
//        if from.src_closed && !from.dest_closed {
//            if from.bytes_available() == 0 {
//                dest_stream.shutdown(Shutdown::Write)?;
//                from.dest_closed = true;
//                return Ok(true);
//            }
//        }
//
//        Ok(false)
//    }
//
//    fn all_closed(&self) -> bool {
//        return self.from_client.src_closed
//            && self.from_client.dest_closed
//            && self.from_server.src_closed
//            && self.from_server.dest_closed;
//    }

    fn channels_on_token(&mut self, token: Token) -> Option<EventChannels> {
        let channels = if token == self.client.token {
            EventChannels {
                from_event: &mut self.from_client,
                from_other: &mut self.from_server,
                event_stream: &mut self.client.stream,
                other_stream: &mut self.server.stream,
                event_token: self.client.token,
                other_token: self.server.token,
                closing: &self.closing,
            }
        } else if token == self.server.token {
            EventChannels {
                from_event: &mut self.from_server,
                from_other: &mut self.from_client,
                event_stream: &mut self.server.stream,
                other_stream: &mut self.client.stream,
                event_token: self.server.token,
                other_token: self.client.token,
                closing: &self.closing,
            }
        } else {
            return None;
        };

        Some(channels)
    }
}

pub enum ConnectionResult<'a> {
    Continue(TokenReady<'a>, TokenReady<'a>),
    Close,
}
