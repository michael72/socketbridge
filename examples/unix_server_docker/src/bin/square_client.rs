// A simple client to send numbers to the square server and print the results.
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: square_client <SOCKET-PATH> <NUMBER1> <NUMBER2> ...");
        return;
    }

    let socket_path = args[1].clone();
    let numbers = &args[2..];

    // Connect to the UNIX socket.
    let mut stream = UnixStream::connect(socket_path).expect("Failed to connect to the server");

    for number in numbers {
        // Send the number to the server.
        if let Err(e) = stream.write_all(format!("{}\n", number).as_bytes()) {
            eprintln!("Failed to send number {}: {}", number, e);
            continue;
        }

        // Read the response from the server.
        let mut reader = BufReader::new(&stream);
        let mut response = String::new();

        if let Ok(_) = reader.read_line(&mut response) {
            println!("{} -> {}", number, response.trim());
        } else {
            eprintln!("Failed to read response for number {}", number);
        }
    }
}
