use mio::net::TcpListener;
use mio::{Poll, Token, Ready, PollOpt, Events};

fn main() {
    let address = "0.0.0.0:8080";
    let listener = TcpListener::bind(&address.parse().unwrap()).unwrap();

    let poll = Poll::new().unwrap();
    poll.register(
        &listener,
        Token(0),
        Ready::readable(),
        PollOpt::edge()).unwrap();

    let mut events = Events::with_capacity(1024);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in &events {
            match event.token() {
                Token(0) => {
                    loop {
                        match listener.accept() {
                            Ok((_socket, address)) => {
                                // Drop the incoming connection just after accepting
                                println!("Connected: {}", address);
                            },
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock =>
                                break,
                            Err(e) =>
                                panic!("Unexpected error: {}", e)
                        }
                    }
                },
                _ => ()
            }
        }
    }
}
