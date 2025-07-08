use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;
use std::time::Duration;

fn send_file(mut write_stream: TcpStream, filepath: &str) -> io::Result<()> {
    let contents = std::fs::read(filepath)?;

    let filename = match filepath.rsplit('/').next() {
        Some(name) => name,
        None => return Ok(()),
    };

    write_stream.write_all(filename.as_bytes())?;
    thread::sleep(Duration::from_millis(50));

    write_stream.write_all(&contents)?;
    Ok(())
}

fn receive_file(mut read_stream: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 1024];

    let read_size = read_stream.read(&mut buffer)?;
    if read_size == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Connection closed",
        ));
    }

    let filename = String::from_utf8_lossy(&buffer[..read_size])
        .trim()
        .to_string();
    let mut store_filename = filename.to_string();
    let mut curr_value = 1;

    while Path::new(&store_filename).exists() {
        store_filename = format!("{filename}_{curr_value}");
        curr_value += 1;
    }

    println!("Receiving file: {filename}");

    let mut file = File::create(&store_filename)?;
    loop {
        let bytes_read = read_stream.read(&mut buffer)?;
        if bytes_read == 0 {
            println!("Connection closed.");
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
    }

    println!("Saved file as: {store_filename}");
    Ok(())
}

fn server_fn(addr: &str, filepath: &str) -> io::Result<()> {
    if !(Path::new(&filepath).exists()) {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "File not found",
        ));
    }

    let listener = TcpListener::bind(addr)?;
    println!("Server running: {addr}");

    for connection_stream in listener.incoming() {
        let handled_stream = connection_stream?;
        println!("New Connection: {}", handled_stream.peer_addr()?);
        // send_file(handled_stream, filepath)?;
        let file_path_clone = filepath.to_string();
        thread::spawn(move || {
            if let Err(e) = send_file(handled_stream, &file_path_clone) {
                eprintln!("Error: {e}");
            }
        });
    }
    Ok(())
}

fn client_fn(addr: &str) -> io::Result<()> {
    let stream = TcpStream::connect(addr)?;
    println!("Connected to server: {addr}");
    receive_file(stream)?;

    Ok(())
}

// usage server: fileshare_cli ip_address port_address filelocation
// usage client: fileshare_cli ip_address port_address

fn main() {
    let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);

    if args.len() == 3 || args.len() == 4 {
        let addr = format!("{}:{}", args[1], args[2]);

        let result = if args.len() == 4 {
            server_fn(&addr, &args[3])
        } else {
            client_fn(&addr)
        };
        if let Err(e) = result {
            eprintln!("Error: {e}");
        }
    } else {
        eprintln!("Server Usage: binary ip_address port_address file_path");
        eprintln!("Client Usage: binary ip_address port_address");
        std::process::exit(1);
    }
}
