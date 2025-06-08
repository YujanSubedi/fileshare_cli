use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

fn file_exists(filename: &str) -> bool {
    Path::new(filename).exists()
}

fn send_file(stream: TcpStream, filepath: &String) -> io::Result<()> {
    let mut write_stream = stream;

    let mut file = File::open(filepath)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    write_stream.write_all(filepath.as_bytes())?;

    write_stream.write_all(&contents)?;
    Ok(())
}

fn receive_file(stream: TcpStream) -> io::Result<()> {
    let mut read_stream = stream;
    let mut buffer = [0; 512];

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
    let mut store_filename = format!("{}", filename);
    let mut curr_value = 1;

    while file_exists(&store_filename) {
        store_filename = format!("{}_{}", filename, curr_value);
        curr_value += 1;
    }

    println!("Receiving file: {}", filename);

    let mut file = File::create(&store_filename)?;
    loop {
        let bytes_read = read_stream.read(&mut buffer)?;
        if bytes_read == 0 {
            println!("Connection closed.");
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
    }

    println!("Saved file as: {}", store_filename);
    Ok(())
}

fn server_fn(addr: &str, filepath: String) -> io::Result<()> {
    let listener = TcpListener::bind(&addr)?;
    println!("Server running: {}", addr);

    for connection_stream in listener.incoming() {
        match connection_stream {
            Ok(stream) => {
                println!("New Connection: {}", stream.peer_addr()?);
                send_file(stream, &filepath)?;
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
}

fn client_fn(addr: &str) -> io::Result<()> {
    let stream = TcpStream::connect(addr)?;
    println!("Connected to server: {}", addr);
    receive_file(stream)?;

    Ok(())
}

// usage sft ip_address port_address filelocation

fn main() {
    let args: Vec<String> = env::args().collect();

    // println!("{:?}", args);

    if args.len() > 2 && args.len() < 5 {
        let ip_address: String = args[1].clone();
        let port_address: String = args[2].clone();
        let addr = format!("{}:{}", ip_address, port_address);
        let result = if args.len() == 4 {
            server_fn(&addr, args[3].clone())
        } else {
            client_fn(&addr)
        };
        if let Err(e) = result {
            eprintln!("Error: {}", e);
        }
    } else {
        println!("Usage binary ip_address port_address [filelocation]");
    }
}
