extern crate net2;

use std::str;
use std::env;
use std::io;
use std::io::prelude::*;
use std::thread;
use std::thread::{JoinHandle};
use std::process::{Command, Stdio};
use std::net::{TcpStream, SocketAddr, Shutdown};
use net2::{TcpBuilder}; //I'm using the TcpBuilder in order to set ipv6_only = false (right now you can't through the TcpListener api)

fn relay_stream_async<F: Read + std::marker::Send + 'static, T: Write + std::marker::Send + 'static>(mut from: F, mut to: T) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer: [u8; 256] = [0; 256];
        loop {
            match from.read(&mut buffer) { //read some
                Ok(amount_read) => {
                    if amount_read == 0 { return; } //stream must be closed
                    match to.write_all(&buffer[..amount_read]) { //successfully read, echo it to [to]
                        Ok(_) => to.flush().is_err(), //successfully echo'd, flush [to]
                        Err(_) => { return; } //stream must be closed
                    };
                },
                Err(_) => { return; } //stream must be closed
            };
        }
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let is_server = args.len() == 1;

    if is_server {
        let maybe_listener = TcpBuilder::new_v6()
                                .and_then(|b| b.only_v6(false)
                                .and_then(|b| b.bind("[::]:514")
                                .and_then(|b| b.listen(514))));

        let listener = match maybe_listener {
            Ok(listener) => listener,
            Err(err) => {
                println!("Unable to start server: {}", err);
                return;
            }
        };

        println!("Listening on: {}", listener.local_addr().unwrap());
        loop {
            let (stream, peer_addr) = listener.accept().unwrap();
            println!("Connected to: {}", peer_addr);

            let mut process = Command::new("cmd")
                                      .arg(format!("/K set ip=[{}]:8080\n", peer_addr.ip())) //set the "ip" env variable to the client's ip 
                                      .stdin(Stdio::piped())
                                      .stdout(Stdio::piped())
                                      .stderr(Stdio::piped())
                                      .spawn()
                                      .unwrap();

            relay_stream_async(process.stderr.take().unwrap(), stream.try_clone().unwrap());
            relay_stream_async(process.stdout.take().unwrap(), stream.try_clone().unwrap());
            let handle = relay_stream_async(stream.try_clone().unwrap(), process.stdin.take().unwrap());
            let _ = handle.join();
            let _ = stream.shutdown(Shutdown::Both);
            println!("Client disconnected");
        }
    } else {
        let addr: SocketAddr = match args[1].parse() {
            Ok(addr) => addr,
            Err(_) => {
                println!("Invalid ip address");
                return;
            }
        };
        
        let stream = match TcpStream::connect(addr) {
            Ok(stream) => stream,
            Err(err) => {
                println!("Could not establish connection: {}", err);
                return;
            }
        };
        
        println!("Connected");

        relay_stream_async(io::stdin(), stream.try_clone().unwrap());
        let handle = relay_stream_async(stream, io::stdout());
        let _ = handle.join();
        println!("Disconnected");
    }
}