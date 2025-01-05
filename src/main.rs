use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;

// Function to transfer data from a reader to a writer in a separate thread.
// Reads data from the reader and writes it to the writer until EOF or an error occurs.
fn transfer_data<R, W>(mut reader: R, mut writer: W)
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = [0; 4096];
        while let Ok(bytes_read) = reader.read(&mut buffer) {
            if bytes_read == 0 {
                break; // EOF reached
            }
            if let Err(e) = writer.write_all(&buffer[..bytes_read]) {
                eprintln!("Error writing to stream: {}", e);
                break;
            }
        }
    })
    .join()
    .unwrap();
}

// Handles communication from a UNIX stream to a TCP stream.
// Sets up bidirectional forwarding between the two streams.
fn handle_unix_to_tcp(unix_stream: UnixStream, tcp_address: String) {
    let tcp_stream = TcpStream::connect(&tcp_address).expect("Failed to connect to TCP server");
    let unix_stream_read = unix_stream.try_clone().expect("Failed to clone UNIX stream");
    let unix_stream_write = unix_stream;
    let tcp_stream_read = tcp_stream.try_clone().expect("Failed to clone TCP stream");
    let tcp_stream_write = tcp_stream;

    // Forward data between UNIX stream and TCP stream.
    transfer_data(unix_stream_read, tcp_stream_write);
    transfer_data(tcp_stream_read, unix_stream_write);
}

// Handles communication from a TCP stream to a UNIX stream.
// Sets up bidirectional forwarding between the two streams.
fn handle_tcp_to_unix(tcp_stream: TcpStream, unix_path: String) {
    let unix_stream = UnixStream::connect(&unix_path).expect("Failed to connect to UNIX socket");
    let tcp_stream_read = tcp_stream.try_clone().expect("Failed to clone TCP stream");
    let tcp_stream_write = tcp_stream;
    let unix_stream_read = unix_stream.try_clone().expect("Failed to clone UNIX stream");
    let unix_stream_write = unix_stream;

    // Forward data between TCP stream and UNIX stream.
    transfer_data(tcp_stream_read, unix_stream_write);
    transfer_data(unix_stream_read, tcp_stream_write);
}

// Runs the application in UNIX mode, setting up a UNIX socket server and forwarding connections to a TCP address.
// If the UNIX socket file already exists, it is removed to avoid binding errors.
fn run_unix_mode(unix_path: String, tcp_address: String) {
    // Ensure the UNIX socket file does not already exist.
    if fs::metadata(&unix_path).is_ok() {
        fs::remove_file(&unix_path).expect("Failed to remove existing UNIX socket file");
    }

    let listener = UnixListener::bind(unix_path.clone()).expect("Failed to bind to UNIX socket");
    println!("UNIX server listening on {}", unix_path);

    for stream in listener.incoming() {
        match stream {
            Ok(unix_stream) => {
                let tcp_address = tcp_address.clone();
                thread::spawn(move || {
                    handle_unix_to_tcp(unix_stream, tcp_address);
                });
            }
            Err(e) => {
                eprintln!("Error accepting UNIX connection: {}", e);
            }
        }
    }
}

// Runs the application in TCP mode, setting up a TCP server and forwarding connections to a UNIX socket.
fn run_tcp_mode(tcp_port: String, unix_path: String) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", tcp_port)).expect("Failed to bind to TCP port");
    println!("TCP server listening on port {}", tcp_port);

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                let unix_path = unix_path.clone();
                thread::spawn(move || {
                    handle_tcp_to_unix(tcp_stream, unix_path);
                });
            }
            Err(e) => {
                eprintln!("Error accepting TCP connection: {}", e);
            }
        }
    }
}

// Main entry point of the application.
// The application bridges UNIX and TCP sockets based on the specified mode ('unix' or 'tcp').
// Usage:
//   unix <UNIX_SOCKET_PATH> <TCP_ADDRESS> - Creates a UNIX socket and forwards data to a TCP address.
//   tcp <TCP_ADDRESS> <UNIX_SOCKET_PATH> - Creates a TCP server and forwards data to a UNIX socket.
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage:");
        eprintln!("  unix <UNIX_SOCKET_PATH> <TCP_ADDRESS>");
        eprintln!("  tcp <TCP_ADDRESS> <UNIX_SOCKET_PATH>");
        return;
    }

    match args[1].as_str() {
        "unix" => {
            let unix_path = args[2].clone();
            let tcp_address = args[3].clone();
            run_unix_mode(unix_path, tcp_address);
        }
        "tcp" => {
            let tcp_port = args[2].clone();
            let unix_path = args[3].clone();
            run_tcp_mode(tcp_port, unix_path);
        }
        _ => {
            eprintln!("Invalid mode. Use 'unix' or 'tcp'.");
        }
    }
}
