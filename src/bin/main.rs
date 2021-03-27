use std::net::TcpListener;
use std::net::TcpStream;

use oly::ThreadPool;


fn dispatch_worker_number(stream : &TcpStream, max: usize) -> usize {
        let peer = stream.peer_addr().unwrap();
        let peer_str = peer.to_string();
        let mut sum: usize = 0;
        for n in peer_str.as_bytes() {
            sum = sum ^ usize::from(*n);
        }
        sum = sum % max;
        sum
}

fn main() {
    let n_workers = 2;
    let pool = ThreadPool::new(n_workers);
    let server = TcpListener::bind("127.0.0.1:9999").unwrap();
    for stream in server.incoming() {
        let inc = stream.unwrap();
        let no = dispatch_worker_number(&inc, n_workers);
        pool.submit(inc, no);
    }
}
