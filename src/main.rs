#![feature(lookup_host)]
#![feature(ip_constructors)]
#![feature(match_default_bindings)]

extern crate mio;

mod channel;
mod token_gen;
mod network_params;
mod port_parser;
mod connection;

use token_gen::TokenGen;
use network_params::get_network_params;
use connection::*;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use std::clone::Clone;
use std::env;
use std::process::exit;
use std::net::{lookup_host, SocketAddr, IpAddr, Ipv4Addr, LookupHost};

use mio::*;
use mio::Poll;
use mio::net::{TcpListener, TcpStream};

const EXIT_FAILURE: i32 = 1;

const MAX_CONNECTION_COUNT: usize = 1024;

fn sock_addr_ip_unspecified(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(<Ipv4Addr>::unspecified()), port)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        println!("Usage: ./net7 LPORT RHOST RPORT");
        exit(EXIT_FAILURE);
    }

    let l_port = &args[1];
    let r_host = &args[2];
    let r_port = &args[3];

    let (l_port, mut r_host, r_port) = match get_network_params(l_port, r_host, r_port) {
        Ok(params) => params,
        Err(e) => {
            eprintln!("{}", e);
            exit(EXIT_FAILURE);
        }
    };

    let r_host = r_host.next().unwrap();

    let localaddr = sock_addr_ip_unspecified(l_port);
    let listener = match TcpListener::bind(&localaddr) {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to bind server socket: {}", e);
            exit(EXIT_FAILURE);
        }
    };

    let poll = Poll::new()
        .expect("Fatal error: failed to init poll");

    let mut token_gen = TokenGen::new();
    let server_token = token_gen.next_token();

    poll.register(&listener, server_token, Ready::readable(), PollOpt::level())
        .expect("Fatal error: failed to register server socket");

    let mut events = Events::with_capacity(MAX_CONNECTION_COUNT);

    let mut token_connections: HashMap<Token, Rc<RefCell<Connection>>> = HashMap::new();
    let mut token_streams: HashMap<Token, TcpStream> = HashMap::new();

    loop {
        match poll.poll(&mut events, None) {
            Ok(event_count) => event_count,
            Err(e) => {
                eprintln!("Poll error: {}", e);
                exit(EXIT_FAILURE);
            }
        };

        for event in events.iter() {
            let token = event.token();
            if token == server_token {
                let client: TcpStream = match listener.accept() {
                    Ok(result) => result.0,
                    Err(e) => {
                        eprintln!("Accept client error: {}", e);
                        continue;
                    }
                };

                let server: TcpStream = match TcpStream::connect(&r_host) {
                    Ok(stream) => stream,
                    Err(e) => {
                        eprintln!("Server connection error: {}", e);
                        continue;
                    }
                };

                let client_token = token_gen.next_token();
                let server_token = token_gen.next_token();

                let mut connection = Connection::new(
                    TokenStream {
                        stream: client,
                        token: client_token,
                    },
                    TokenStream {
                        stream: server,
                        token: server_token,
                    },
                );

                let rc_connection = Rc::new(RefCell::new(connection));
                token_connections.insert(client_token, Rc::clone(&rc_connection));
                token_connections.insert(server_token, Rc::clone(&rc_connection));
//                let streams = &rc_connection.borrow().streams;
//
//                poll.register(streams[0].stream, client_token,
//                              Ready::readable(), PollOpt::level())
//                    .expect("Failed to register");
//
//                poll.register(streams[1].stream, server_token,
//                              Ready::readable(), PollOpt::level())
//                    .expect("Failed to register");
            } else {
                let mut connection = token_connections[&token].borrow_mut();
                connection.handle_event(token, event);
            }
        }
    }
}
