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

#[derive(FromArgs)]
/// Runs a benchmarking server over the melnet2 transport network.
struct ClientArgs {
    /// where to connect to (ip:port)
    #[argh(option)]
    connect: SocketAddr,

    /// how many connections to start (default: 16)
    #[argh(option, default = "16")]
    conns: usize,
}

fn main() {
    let args: ClientArgs = argh::from_env();
    // instead of using the melnet2/nanorpc crates and an async executor, we manually open connections and manually spam requests using highly efficient synchronous code. melnet2 is pipelined so we can achieve extremely high RPS without massive concurrency
    // this ensures that we are not benchmarking the async executor, epoll wrapper, etc, and that the server is the bottleneck. it also serves as an example of how to write a benchmarking tool in another language.

    // right now all we spam is the "hello world" request, lol
    let counter = Arc::new(AtomicUsize::new(0));
    for _ in 0..args.conns {
        let counter = counter.clone();
        std::thread::spawn(move || spam_reqs(&counter, args.connect));
    }
    loop {
        let before = counter.load(Ordering::Relaxed);
        std::thread::sleep(Duration::from_secs(1));
        let after = counter.load(Ordering::Relaxed);
        eprintln!("{} requests/sec", after - before);
    }
}

fn spam_reqs(counter: &AtomicUsize, dest: SocketAddr) {
    let conn = TcpStream::connect(dest).expect("connection failed");
    let mut upstream = conn.try_clone().expect("cannot clone tcp conn");
    // one thread spams the same request over and over
    std::thread::spawn(move || {
        let req =
            b"{\"jsonrpc\": \"2.0\", \"method\": \"subtract\", \"params\": [42, 23], \"id\": 1}\n";
        loop {
            upstream.write_all(req).expect("cannot write");
        }
    });
    // the other thread reads responses over and over. we use a bufreader to read line by line, but that should not be the bottleneck
    let mut downstream = BufReader::new(conn);
    let mut line = String::new();
    loop {
        line.clear();
        downstream.read_line(&mut line).expect("cannot read");
        counter.fetch_add(1, Ordering::Relaxed);
    }
}
