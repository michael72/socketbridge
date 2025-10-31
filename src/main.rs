use clap::{Arg, Command};
use std::fs;
use std::io::{self, Read, Write};
use std::net::ToSocketAddrs;
use std::net::{TcpListener, TcpStream};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;

#[derive(Debug)]
enum BridgeError {
    IoError(io::Error),
    Eof, // Custom error for EOF
}

impl From<io::Error> for BridgeError {
    fn from(err: io::Error) -> Self {
        BridgeError::IoError(err)
    }
}

// Function to transfer data from a client channel to a server channel.
// The expected underlying protocol is that the client will send data to the server
// and the server will exactly respond once to the client until the client disconnects.
fn bridge_client_server<C1, C2>(
    client: &mut C1,
    server: &mut C2,
    buffer_size: usize,
) -> Result<(), BridgeError>
where
    C1: Read + Write,
    C2: Read + Write,
{
    let mut buffer = vec![0; buffer_size];
    let mut data_available = false;

    // Read request from the client (loop if buffer is filled completely)
    loop {
        let bytes_read = client.read(&mut buffer)?;
        if bytes_read == 0 {
            // EOF or client closed connection - only return error if no data was read
            if !data_available {
                return Err(BridgeError::Eof);
            }
            break;
        }

        data_available = true;

        // Forward data to the server immediately
        server.write_all(&buffer[..bytes_read])?;

        // If we read less than buffer_size, we've read all available data
        if bytes_read < buffer_size {
            break;
        }
    }

    server.flush()?;

    data_available = false;

    // Read response from the server (loop if buffer is filled completely)
    loop {
        let bytes_read = server.read(&mut buffer)?;
        if bytes_read == 0 {
            // EOF or server closed connection - only return error if no data was read
            if !data_available {
                return Err(BridgeError::Eof);
            }
            break;
        }

        data_available = true;

        // Send data back to the client immediately
        client.write_all(&buffer[..bytes_read])?;

        // If we read less than buffer_size, we've read all available data
        if bytes_read < buffer_size {
            break;
        }
    }

    client.flush()?;

    Ok(())
}

// Handles communication from a UNIX stream to a TCP stream.
// Sets up unidirectional forwarding from client to server and back.
fn handle_unix_to_tcp(mut unix_stream: UnixStream, tcp_address: String, buffer_size: usize) {
    let mut tcp_stream = TcpStream::connect(&tcp_address).expect("Failed to connect to TCP server");

    loop {
        match bridge_client_server(&mut unix_stream, &mut tcp_stream, buffer_size, 1) {
            Ok(_) => {} // Continue the loop on successful communication
            Err(BridgeError::Eof) => {
                // Break on EOF without logging an error
                println!("Connection closed by client or server.");
                break;
            }
            Err(BridgeError::IoError(e)) => {
                // Log other I/O errors and break
                eprintln!("Error in client-server communication: {}", e);
                break;
            }
        }
    }
}

// Handles communication from a TCP stream to a UNIX stream.
// Sets up bidirectional forwarding between the two streams.
fn handle_tcp_to_unix(mut tcp_stream: TcpStream, unix_path: String, buffer_size: usize) {
    let mut unix_stream =
        UnixStream::connect(&unix_path).expect("Failed to connect to UNIX socket");

    loop {
        match bridge_client_server(&mut tcp_stream, &mut unix_stream, buffer_size, 2) {
            Ok(_) => {} // Continue the loop on successful communication
            Err(BridgeError::Eof) => {
                // Break on EOF without logging an error
                println!("Connection closed by client or server.");
                break;
            }
            Err(BridgeError::IoError(e)) => {
                // Log other I/O errors and break
                eprintln!("Error in client-server communication: {}", e);
                break;
            }
        }
    }
}

// Runs the application in UNIX mode, setting up a UNIX socket server and forwarding connections to a TCP address.
// If the UNIX socket file already exists, it is removed to avoid binding errors.
fn run_unix_mode(unix_path: String, tcp_address: String, buffer_size: usize) {
    // Ensure the UNIX socket file does not already exist.
    if fs::metadata(&unix_path).is_ok() {
        fs::remove_file(&unix_path).expect("Failed to remove existing UNIX socket file");
    }

    let listener = UnixListener::bind(unix_path.clone()).expect("Failed to bind to UNIX socket");
    println!(
        "UNIX server listening on {} (buffer size: {})",
        unix_path, buffer_size
    );

    for stream in listener.incoming() {
        match stream {
            Ok(unix_stream) => {
                println!("bridge: unix client connected");
                let tcp_address = tcp_address.clone();
                thread::spawn(move || {
                    handle_unix_to_tcp(unix_stream, tcp_address, buffer_size);
                });
            }
            Err(e) => {
                eprintln!("Error accepting UNIX connection: {}", e);
            }
        }
    }
}

// Runs the application in TCP mode, setting up a TCP server and forwarding connections to a UNIX socket.
fn run_tcp_mode(tcp_address: String, unix_path: String, buffer_size: usize) {
    let listener = TcpListener::bind(tcp_address.clone()).expect("Failed to bind to TCP port");
    println!(
        "TCP server listening on port {} (buffer size: {})",
        tcp_address, buffer_size
    );

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                println!("bridge: tcp client connected");
                let unix_path = unix_path.clone();
                thread::spawn(move || {
                    handle_tcp_to_unix(tcp_stream, unix_path, buffer_size);
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
    let matches = Command::new("socketbridge")
        .about("Bridges UNIX and TCP sockets")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("unix")
                .about("Create a UNIX socket server and forward to a TCP address")
                .arg(
                    Arg::new("unix_path")
                        .help("Path to the UNIX socket")
                        .num_args(1)
                        .required(true),
                )
                .arg(
                    Arg::new("tcp_address")
                        .help("TCP address to forward to (e.g., 127.0.0.1:1234)")
                        .num_args(1)
                        .required(true)
                        .value_parser(validate_tcp_address),
                )
                .arg(
                    Arg::new("buffer_size")
                        .short('b')
                        .long("buffer-size")
                        .help("Buffer size for data transfer")
                        .default_value("4096")
                        .value_parser(clap::value_parser!(usize)),
                ),
        )
        .subcommand(
            Command::new("tcp")
                .about("Create a TCP server and forward to a UNIX socket")
                .arg(
                    Arg::new("tcp_address")
                        .help("TCP address to bind to (e.g., 0.0.0.0:1234)")
                        .num_args(1)
                        .required(true)
                        .value_parser(validate_tcp_address),
                )
                .arg(
                    Arg::new("unix_path")
                        .help("Path to the UNIX socket")
                        .num_args(1)
                        .required(true),
                )
                .arg(
                    Arg::new("buffer_size")
                        .short('b')
                        .long("buffer-size")
                        .help("Buffer size for data transfer")
                        .default_value("4096")
                        .value_parser(clap::value_parser!(usize)),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("unix", sub_m)) => {
            let unix_path = sub_m.get_one::<String>("unix_path").unwrap().clone();
            let tcp_address = sub_m.get_one::<String>("tcp_address").unwrap().clone();
            let buffer_size = *sub_m.get_one::<usize>("buffer_size").unwrap();
            run_unix_mode(unix_path, tcp_address, buffer_size);
        }
        Some(("tcp", sub_m)) => {
            let tcp_address = sub_m.get_one::<String>("tcp_address").unwrap().clone();
            let unix_path = sub_m.get_one::<String>("unix_path").unwrap().clone();
            let buffer_size = *sub_m.get_one::<usize>("buffer_size").unwrap();
            run_tcp_mode(tcp_address, unix_path, buffer_size);
        }
        _ => eprintln!("Invalid command"),
    }
}

fn validate_tcp_address(addr: &str) -> Result<String, String> {
    // Parse the address and ensure it's valid
    addr.to_socket_addrs()
        .map(|_| addr.to_string())
        .map_err(|_| format!("Invalid TCP address: {}", addr))
}
