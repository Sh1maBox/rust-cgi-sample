use std::io;
use std::io::prelude::*;
use std::io::stdin;
use std::net::TcpListener;
use std::net::TcpStream;
use std::os::fd::AsRawFd;

use nix::sys::wait::waitpid;
use nix::unistd::execv;
use nix::unistd::pipe;
use nix::unistd::read;
use nix::unistd::write;
use nix::unistd::{dup2, fork, ForkResult};
use std::ffi::CString;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();

    let cgi_path = std::env::var("CGI_BINARY_PATH").unwrap();

    let handler = Handler::new(cgi_path);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handler.handle_connection(stream)
    }
}

struct Handler {
    pub cgi_path: String,
}

impl Handler {
    fn new(cgi_path: String) -> Self {
        Self { cgi_path }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];

        stream.read(&mut buffer).unwrap();

        println!("{}", String::from_utf8_lossy(&buffer[..]));

        let (reader, writer) = pipe().unwrap();
        let (reader2, writer2) = pipe().unwrap();

        match unsafe { fork() }.expect("Fork failed") {
            ForkResult::Parent { child } => {
                println!(
                    "Continuing execution in parent process, new child has pid: {}",
                    child
                );

                write(writer, b"parent: Hello World").unwrap();

                while let Ok(status) = waitpid(child, None) {
                    println!("Reaped child. Status: {:?}", status);
                }

                let mut buffer = [0; 1024];
                let n = read(reader2.as_raw_fd(), &mut buffer).unwrap();

                println!("Parent received: {}", String::from_utf8_lossy(&buffer[..n]));
                stream.write(&buffer[..n]).unwrap();
            }
            ForkResult::Child => {
                println!("I'm a new child process");

                // let mut buffer = [0; 1024];
                // let _ = read(reader.as_raw_fd(), &mut buffer);

                let _ = dup2(reader.as_raw_fd(), stdin().as_raw_fd());

                let _ = dup2(writer2.as_raw_fd(), io::stdout().as_raw_fd());

                let bin_name: CString = CString::new(self.cgi_path.clone()).unwrap();

                let _ = execv::<CString>(&bin_name, &[]);
            }
        }
    }
}
