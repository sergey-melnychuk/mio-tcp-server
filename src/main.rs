use mio::net::{TcpListener, TcpStream};
use mio::{Poll, Token, Ready, PollOpt, Events};
use std::collections::HashMap;
use std::io::Read;

fn main() {
    let address = "0.0.0.0:8080";
    let listener = TcpListener::bind(&address.parse().unwrap()).unwrap();

    let poll = Poll::new().unwrap();
    poll.register(
        &listener,
        Token(0),
        Ready::readable(),
        PollOpt::edge()).unwrap();

    let mut sockets: HashMap<Token, TcpStream> = HashMap::new();
    let mut counter: usize = 0;
    let mut buffer = [0 as u8; 1024];

    let mut events = Events::with_capacity(1024);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in &events {
            match event.token() {
                Token(0) => {
                    loop {
                        match listener.accept() {
                            Ok((socket, _)) => {
                                counter += 1;
                                let token = Token(counter);

                                poll.register(
                                    &socket,
                                    token,
                                Ready::readable(),
                                PollOpt::edge()).unwrap();

                                sockets.insert(token, socket);
                            },
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock =>
                                break,
                            Err(e) =>
                                panic!("Unexpected error: {}", e)
                        }
                    }
                },
                token if event.readiness().is_readable() => {
                    let mut bytes_read: usize = 0;
                    loop {
                        let read = sockets.get_mut(&token).unwrap().read(&mut buffer);
                        match read {
                            Ok(0) => {
                                sockets.remove(&token);
                                break;
                            },
                            Ok(n) => {
                                println!("Read {} bytes: {:?}", n, &buffer[0..n]);
                                bytes_read += n;
                            },
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock =>
                                break,
                            Err(e) =>
                                panic!("Unexpected error: {}", e)
                        }
                    }
                    println!("Read {} bytes for token {}", bytes_read, token.0);
                }
                _ => ()
            }
        }
    }
}
