// A simple UNIX socket server that squares received numbers and sends the result back.
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::fs;

fn main() {
    let socket_path = "/tmp/square_server.sock";

    // Remove existing socket file if it exists.
    if fs::metadata(socket_path).is_ok() {
        fs::remove_file(socket_path).expect("Failed to remove existing socket file");
    }

    // Create a UNIX socket listener.
    let listener = UnixListener::bind(socket_path).expect("Failed to bind to UNIX socket");
    println!("Square server is running at {}", socket_path);

    for stream in listener.incoming() {
        match stream {
            Ok(mut client) => {
                println!("Client connected");

                let mut reader = BufReader::new(client.try_clone().expect("Failed to clone client stream"));
                let mut buffer = String::new();

                while let Ok(bytes_read) = reader.read_line(&mut buffer) {
                    if bytes_read == 0 {
                        break; // EOF
                    }

                    // Parse the input as a number.
                    if let Ok(number) = buffer.trim().parse::<i32>() {
                        let squared = number * number;
                        let response = format!("{}\n", squared);

                        if let Err(e) = client.write_all(response.as_bytes()) {
                            eprintln!("Error sending response: {}", e);
                            break;
                        }
                        /*if let Err(e) = client.flush() {
                            eprintln!("Error flushing stream: {}", e);
                        }*/
                    } else {
                        eprintln!("Invalid number received: {}", buffer.trim());
                    }

                    buffer.clear();
                }

                println!("Client disconnected");
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

