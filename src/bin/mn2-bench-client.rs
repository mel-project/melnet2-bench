use std::{
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpStream},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use argh::FromArgs;

const HELLO_WORLD_REQUEST: &[u8;67] =
b"{\"jsonrpc\": \"2.0\", \"method\": \"hello-world\", \"params\": [], \"id\": 1}\n";

// use serde::{Deserialize, Serialize};

// #[derive(Serialize, Deserialize)]
// struct Request {
//     jsonrpc: String,
//     method: String,
//     params: Vec<String>,
//     id: usize,
// }

// trait RequestHelpers {
//     fn send(&mut self, input_struct: &Request);
// }
//
// impl RequestHelpers for TcpStream {
//     fn send(&mut self, input_struct: &Request) {
//         let input_string: String = serde_json::to_string(&input_struct).expect("Could not convert struct to string.");
//
//         self.write(input_string.as_bytes()).expect("Could not write input string to stream");
//         self.write(b"\n").expect("Could not write newline to stream");
//         self.flush().expect("Could not flush stream.");
//     }
// }

#[derive(FromArgs)]
/// Runs a benchmarking server over the melnet2 transport network.
struct ClientArgs {
    /// where to connect to (ip:port)
    #[argh(option)]
    connect: SocketAddr,

    /// how many connections to start (default: 16)
    #[argh(option, default = "16")]
    connections: usize,
}

fn spam_reqs(counter: &AtomicUsize, dest: SocketAddr) {
    let connection: TcpStream = TcpStream::connect(dest).expect("connection failed");
    
    let mut upstream: TcpStream = connection.try_clone().expect("cannot clone tcp connection");

    // let request: Request = Request {
    //     jsonrpc: String::from("2.0"),
    //     method: String::from("hello-world"),
    //     params: vec![String::new()],
    //     id: 1,
    // };

    // one thread spams the same request over and over
    std::thread::spawn(move || {
        // loop {
        //     upstream.send(&request);
        // }

        loop {
            upstream.write_all(HELLO_WORLD_REQUEST).expect("cannot write");
        }
    });
    
    // the other thread reads responses over and over. we use a bufreader to read line by line, but
    // that should not be the bottleneck
    let mut downstream: BufReader<TcpStream> = BufReader::new(connection);

    let mut line: String = String::new();

    loop {
        line.clear();

        downstream.read_line(&mut line).expect("cannot read");

        counter.fetch_add(1, Ordering::Relaxed);
    }
}

fn main() {
    let args: ClientArgs = argh::from_env();
    // instead of using the melnet2/nanorpc crates and an async executor, we manually open
    // connections and manually spam requests using highly efficient synchronous code.
    // melnet2 is pipelined so we can achieve extremely high RPS without massive concurrency.
    // This ensures that we are not benchmarking the async executor, epoll wrapper, etc, and that
    // the server is the bottleneck. it also serves as an example of how to write a benchmarking
    // tool in another language.

    // right now all we spam is the "hello world" request, lol
    let counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    let range: std::ops::Range<usize> = 0..args.connections;

    range.into_iter().for_each(|_index| {
        let counter = counter.clone();
        std::thread::spawn(move || spam_reqs(&counter, args.connect));
    });

    loop {
        let before: usize = counter.load(Ordering::Relaxed);

        std::thread::sleep(Duration::from_secs(1));

        let after: usize = counter.load(Ordering::Relaxed);

        eprintln!("{} requests/sec", after - before);
    }
}