use std::io::prelude::*;
use std::io;
use std::process::{Command, Stdio};
use std::net::{TcpListener, TcpStream, SocketAddr, Shutdown};
use std::str;
use std::thread;
use std::thread::{JoinHandle};
use std::env;

fn handle_outstream_async<F: Read + std::marker::Send + 'static, T: Write + std::marker::Send + 'static>(mut from: F, mut to: T) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer: [u8; 256] = [0; 256];
        loop {
            match from.read(&mut buffer) { //read some
                Ok(amount_read) => {
                    if amount_read == 0 { return; }
                    match to.write_all(&buffer[..amount_read]) { //successfully read, echo it to [to]
                        Ok(_) => to.flush().is_err(), //successfully echo'd, flush [to]
                        Err(_) => { return; }
                    };
                },
                Err(_) => { return; }
            };
        }
    })
}

fn handle_instream_async<F: Read + std::marker::Send + 'static, T: Write + std::marker::Send + 'static>(mut from: F, mut to: T) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer: [u8; 256] = [0; 256];
        loop {
            match from.read(&mut buffer) { //read some
                Ok(amount_read) => { 
                    if amount_read == 0 { return; }
                    match to.write_all(&buffer[..amount_read]) { //successfully read, echo it to [to]
                        Ok(_) => to.flush().is_err(), //successfully echo'd, flush [to]
                        Err(_) => { return; }
                    };
                },
                Err(_) => { return; }
            };
        }
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let is_server = args.len() == 1;

    if is_server {
        let listener = TcpListener::bind("[::]:514").unwrap();
        println!("Listening on: {}", listener.local_addr().unwrap());
        loop {
            let (stream, peer_addr) = listener.accept().unwrap();
            println!("Connected to: {}", peer_addr);

            let mut process = Command::new("cmd")
                                    .stdin(Stdio::piped())
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::piped())
                                    .spawn()
                                    .unwrap();

            let handle1 = handle_outstream_async(process.stderr.take().unwrap(), stream.try_clone().unwrap());
            let handle2 = handle_outstream_async(process.stdout.take().unwrap(), stream.try_clone().unwrap());
            let handle = handle_instream_async(stream.try_clone().unwrap(), process.stdin.take().unwrap());
            let _ = handle.join();
            let _ = stream.shutdown(Shutdown::Both);
            let _ = handle1.join();
            let _ = handle2.join();
            println!("Client disconnected");
        }
    } else {
        let addr: SocketAddr = args[1].parse().unwrap();
        let stream = TcpStream::connect(addr).unwrap();

        handle_instream_async(io::stdin(), stream.try_clone().unwrap());
        let handle = handle_outstream_async(stream, io::stdout());
        let _ = handle.join();
    }
}