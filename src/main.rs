use anyhow::Result;
use commands::Command;
use connection::{Connection, ConnectionState::*};
use mio::event::Event;
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{self, Read, Write};

pub mod commands;
pub mod connection;
pub mod hashtable;
pub mod scalablehashmap;

const SERVER: Token = Token(0);

fn main() -> Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);
    let addr = "127.0.0.1:6379".parse()?;
    let mut server = TcpListener::bind(addr)?;
    println!("Server started on {}", addr);
    poll.registry()
        .register(&mut server, SERVER, Interest::READABLE)?;
    let mut connections = HashMap::new();
    let mut unique_token = Token(SERVER.0 + 1);

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            match event.token() {
                SERVER => loop {
                    let (mut stream, address) = match server.accept() {
                        Ok((stream, address)) => (stream, address),
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            break;
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    };
                    println!("Accepted connection from: {}", address);
                    let token = next(&mut unique_token);
                    poll.registry().register(
                        &mut stream,
                        token,
                        Interest::READABLE.add(Interest::WRITABLE),
                    )?;
                    let connection = Connection::new(stream);

                    connections.insert(token, connection);
                },
                token => {
                    if let Some(connection) = connections.get_mut(&token) {
                        match connection.state {
                            ReadyToRead => {
                                if event.is_readable() {
                                    read_request(connection, &poll, event)?;
                                }
                            }
                            ReadyToWrite => {
                                if event.is_writable() {
                                    send_response(connection)?
                                }
                            }
                            Closing => {
                                poll.registry().deregister(connection.stream_mut())?;
                                connections.remove(&token);
                            }
                        }
                    };
                }
            }
        }
    }
}

fn read_request(connection: &mut Connection, poll: &Poll, event: &Event) -> Result<()> {
    let mut bytes_read = 0;
    loop {
        match connection
            .stream
            .read(&mut connection.read_buffer[bytes_read..])
        {
            Ok(0) => {
                break;
            }
            Ok(n) => {
                bytes_read += n;
                connection.state = ReadyToWrite;
                let hello_world_vec: Vec<u8> = vec![
                    12, 0, 0, 0, // 4-byte length (12 in big-endian format)
                    104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100,
                    33, // "hello world!"
                ];
                let len = hello_world_vec.len();
                connection.write_buffer[..len].copy_from_slice(&hello_world_vec);
                poll.registry().reregister(
                    &mut connection.stream,
                    event.token(),
                    Interest::WRITABLE,
                )?;
                // connection.reset_read_buffer();
            }
            Err(ref err) if would_block(err) => break,
            Err(ref err) if interrupted(err) => continue,
            Err(err) => return Err(err.into()),
        }
    }
    Ok(if bytes_read != 0 {
        let command = Command::parse_request(&connection.read_buffer)?;
        match command {
            Command::Get(key) => {
                let _ = commands::get::invoke(key);
            }
            Command::Set(key, value) => {
                let _ = commands::set::invoke(key, value);
            }
            Command::Del(key) => {
                let _ = commands::del::invoke(key);
            }
        }
    })
}

fn send_response(connection: &mut Connection) -> Result<()> {
    match connection.stream.write_all(&connection.write_buffer) {
        Ok(_) => Ok({
            connection.state = Closing;
            connection.reset_write_buffer();
        }),
        Err(ref err) if would_block(err) => Ok({}),
        Err(err) => return Err(err.into()),
    }
}

fn next(current: &mut Token) -> Token {
    let next = current.0;
    current.0 += 1;
    Token(next)
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}
