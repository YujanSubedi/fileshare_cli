use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::Command;
use std::thread::{self, JoinHandle};
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

    let store_filename = {
        let mut temp_filename = filename.to_string();
        let mut curr_value = 1;

        while Path::new(&temp_filename).exists() {
            temp_filename = format!("{filename}_{curr_value}");
            curr_value += 1;
        }
        temp_filename
    };

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

fn tcp_server_fn(addr: &str, filepath: &str) -> io::Result<()> {
    if !(Path::new(&filepath).exists()) {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "File not found",
        ));
    }

    let listener = TcpListener::bind(addr)?;
    println!("TCP server: {}", listener.local_addr()?);

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

fn http_server_fn(addr: &str, filepath: &str) -> io::Result<()> {
    let tcp_listener = TcpListener::bind(addr)?;
    println!("HTTP server: http://{}", tcp_listener.local_addr()?);

    let contents = fs::read(filepath)?;
    let filename = match filepath.rsplit('/').next() {
        Some(name) => name,
        None => return Ok(()),
    };

    let mut buf = [0; 1024];
    let get_request = "GET / HTTP/1.1\r\n".as_bytes();
    let response_404 = "HTTP/1.1 404 NOT FOUND\r\n\r\n".as_bytes();
    let response_header = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\n\r\n", contents.len(), filename).into_bytes();

    for stream in tcp_listener.incoming() {
        let mut handled_stream = stream?;
        println!("request from {}", handled_stream.peer_addr()?);
        let _read_byte_size = handled_stream.read(&mut buf)?;
        if buf.starts_with(get_request) {
            handled_stream.write_all(&response_header)?;
            handled_stream.write_all(&contents)?;
        } else {
            handled_stream.write_all(response_404)?;
        }
    }

    Ok(())
}

fn handle_servers(file_path: String, http_port: &str, tcp_port: &str) {
    let ip_addr_cmd = Command::new("sh")
        .arg("-c")
        .arg("ip a | grep global | cut -d ' ' -f 6 | cut -d '/' -f 1")
        .output()
        .expect("Faild to execute command");

    if ip_addr_cmd.status.success() {
        let ip_addrs_vec = {
            let mut temp_vec: Vec<String> = String::from_utf8_lossy(&ip_addr_cmd.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect();
            temp_vec.push("127.0.0.1".to_string());
            temp_vec.push("::1".to_string());
            temp_vec
        };
        println!("Address: {ip_addrs_vec:?}");

        let mut threads_vec: Vec<JoinHandle<()>> = Vec::with_capacity(8);
        for ip_addr in ip_addrs_vec {
            let tcp_addr = format!("{ip_addr}:{tcp_port}");
            let file_path_clone = file_path.clone();
            let tcp_thread_handle = thread::spawn(move || {
                if let Err(e) = tcp_server_fn(&tcp_addr, &file_path_clone) {
                    eprintln!("Error in tcp server {tcp_addr}: {e}");
                };
            });
            threads_vec.push(tcp_thread_handle);

            let http_addr = format!("{ip_addr}:{http_port}");
            let file_path_clone = file_path.clone();
            let http_thread_handle = thread::spawn(move || {
                if let Err(e) = http_server_fn(&http_addr, &file_path_clone) {
                    eprintln!("Error in tcp server {http_addr}: {e}");
                };
            });
            threads_vec.push(http_thread_handle);
        }

        for thread in threads_vec {
            if let Err(e) = thread.join() {
                eprintln!("Error: {e:?}");
            };
        }
    }
}

fn handle_tcp_client(addr: &str) -> io::Result<()> {
    let stream = TcpStream::connect(addr)?;
    println!("Connected to server: {addr}");
    receive_file(stream)?;

    Ok(())
}

// usage server: fileshare_cli filelocation
// usage client: fileshare_cli ip_address port_address

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => {
            let file_path = args[1].to_string();
            handle_servers(file_path, "0", "0");
        }
        4 => {
            let file_path = args[1].to_string();
            let http_port = &args[2];
            let tcp_port = &args[3];
            handle_servers(file_path, http_port, tcp_port);
        }
        3 => {
            let full_addr = format!("{}:{}", &args[1], &args[2]);
            if let Err(e) = handle_tcp_client(&full_addr) {
                eprintln!("Error: {e}");
            };
        }
        _ => {
            eprintln!("USAGE:");
            eprintln!("Server: fileshare_cli filepath");
            eprintln!("Server: fileshare_cli filepath http_port tcp_port ");
            eprintln!("Client: fileshare_cli ip_address port_address");
            std::process::exit(1);
        }
    }
}
