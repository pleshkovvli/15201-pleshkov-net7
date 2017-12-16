extern crate mio;

use channel::Channel;

use mio::net::TcpStream;
use mio::Token;
use mio::Ready;
use mio::Event;

const BUFFER_SIZE: usize = 8192;

type TcpChannel = Channel<TcpStream, TcpStream>;

pub struct Connection {
    client: TokenStream,
    server: TokenStream,
    from_client: TcpChannel,
    from_server: TcpChannel,
}

pub struct TokenStream {
    pub token: Token,
    pub stream: TcpStream,
}

pub struct EventChannels<'a> {
    from: &'a TcpChannel,
    to: &'a TcpChannel,
    event_stream: &'a TcpStream,
    other_stream: &'a TcpStream,
    event_token: Token,
    other_token: Token
}

impl<'a> EventChannels<'a> {
    fn reverse(self) -> EventChannels<'a> {
        let from = self.from;
        self.from = self.to;
        self.to = self.from;
        self
    }
}

pub struct TokenReady {
    token: Token,
    ready: Ready,
}

impl Connection {
    pub fn new(client: TokenStream, server: TokenStream) -> Connection {
        let from_client: TcpChannel = Channel::new(BUFFER_SIZE);
        let from_server: TcpChannel = Channel::new(BUFFER_SIZE);

        Connection { client, server, from_client, from_server }
    }

    fn get_stream_ready(channels: EventChannels) -> Ready {
        let mut ready = Ready::empty();
        if channels.from.free_space() > 0 {
            ready.insert(Ready::readable());
        }
        if channels.to.bytes_available() > 0 {
            ready.insert(Ready::writable());
        }

        ready
    }

    pub fn handle_event(&mut self, token: Token, event: Event) -> Option<(TokenReady, TokenReady)> {
        let channels = self.channels_on_token(token)?;

        self.handle_channels(channels, event.readiness())
    }

    fn handle_channels(&mut self, event_channels: EventChannels, ready: Ready)
                       -> Option<(TokenReady, TokenReady)> {
        if ready.is_readable() {
            event_channels.from.recv_bytes(event_channels.event_stream);
        }
        if ready.is_writable() {
            event_channels.to.send_bytes(event_channels.other_stream);
        }

        Some((
            TokenReady {
                token: event_channels.event_token,
                ready: Self::get_stream_ready(event_channels),
            },
            TokenReady {
                token: event_channels.other_token,
                ready: Self::get_stream_ready(event_channels.reverse()),
            }
        ))
    }

    fn channels_on_token(&self, token: Token) -> Option<EventChannels> {
        let channels = if token == self.client.token {
            EventChannels {
                from: &self.from_client,
                to: &self.from_server,
                event_stream: &self.client.stream,
                other_stream: &self.server.stream,
                event_token: self.client.token,
                other_token: self.server.token,
            }
        } else if token == self.server.token {
            EventChannels {
                from: &self.from_server,
                to: &self.from_client,
                event_stream: &self.server.stream,
                other_stream: &self.client.stream,
                event_token: self.server.token,
                other_token: self.client.token,
            }
        } else {
            return None;
        };

        Some(channels)
    }
}
