// Benchmarks:
// $ ab -n 1000000 -c 128 -k http://127.0.0.1:8080/
// $ wrk -d 30s -t 4 -c 128 http://127.0.0.1:8080/

use mio::net::{TcpListener, TcpStream};
use mio::{Poll, Token, Ready, PollOpt, Events};
use std::collections::HashMap;
use std::io::{Read, Write};

static RESPONSE: &str = "HTTP/1.1 200 OK
Content-Type: text/html
Connection: keep-alive
Content-Length: 6

hello
";

fn is_double_crnl(window: &[u8]) -> bool {
    window.len() >= 4 &&
        (window[0] == '\r' as u8) &&
        (window[1] == '\n' as u8) &&
        (window[2] == '\r' as u8) &&
        (window[3] == '\n' as u8)
}

fn main() {
    let address = "0.0.0.0:9000";
    let listener = TcpListener::bind(&address.parse().unwrap()).unwrap();

    let poll = Poll::new().unwrap();
    poll.register(
        &listener,
        Token(0),
        Ready::readable(),
        PollOpt::edge()).unwrap();

    let mut counter: usize = 0;
    let mut sockets: HashMap<Token, TcpStream> = HashMap::new();
    let mut requests: HashMap<Token, Vec<u8>> = HashMap::new();
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
                                requests.insert(token, Vec::with_capacity(192));
                            },
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock =>
                                break,
                            Err(_) => break
                        }
                    }
                },
                token if event.readiness().is_readable() => {
                    loop {
                        let read = sockets.get_mut(&token).unwrap().read(&mut buffer);
                        match read {
                            Ok(0) => {
                                sockets.remove(&token);
                                break
                            },
                            Ok(n) => {
                                let req = requests.get_mut(&token).unwrap();
                                for b in &buffer[0..n] {
                                    req.push(*b);
                                }
                            },
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock =>
                                break,
                            Err(_) => break
                        }
                    }

                    let ready = requests.get(&token).unwrap()
                        .windows(4)
                        .find(|window| is_double_crnl(*window))
                        .is_some();

                    if ready {
                        let socket = sockets.get(&token).unwrap();
                        poll.reregister(
                            socket,
                            token,
                            Ready::writable(),
                            PollOpt::edge() | PollOpt::oneshot()).unwrap();
                    }
                },
                token if event.readiness().is_writable() => {
                    requests.get_mut(&token).unwrap().clear();
                    sockets.get_mut(&token).unwrap().write_all(RESPONSE.as_bytes()).unwrap();

                    // Re-use existing connection ("keep-alive") - switch back to reading
                    poll.reregister(
                        sockets.get(&token).unwrap(),
                        token,
                        Ready::readable(),
                        PollOpt::edge()).unwrap();
                },
                _ => unreachable!()
            }
        }
    }
}
