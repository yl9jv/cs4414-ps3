//
// zhtta.rs
//
// Running on Rust 0.8
//
// Starting code for PS3
// 
// Note: it would be very unwise to run this server on a machine that is
// on the Internet and contains any sensitive files!
//
// University of Virginia - cs4414 Fall 2013
// Weilin Xu and David Evans
// Version 0.3

extern mod extra;

use std::rt::io::*;
use std::rt::io::net::ip::SocketAddr;
use std::io::println;
use std::cell::Cell;
use std::{os, str, io};
use extra::arc;
use std::comm::*;
use extra::priority_queue;

static PORT:    int = 4414;
static IP: &'static str = "127.0.0.1";

struct sched_msg {
    stream: Option<std::rt::io::net::tcp::TcpStream>,
    filepath: ~std::path::PosixPath,
    topPriority: int,
    fileSize: u64
}

impl Ord for sched_msg {
    fn lt(&self, other: &sched_msg) -> bool {

        let mut retVal:bool = false;

        //If self has a lower geographic priority, then it is less than
        if (self.topPriority < other.topPriority) {
            retVal = true;
        }
        else if (self.topPriority == other.topPriority) {

            //If the two have equal priorities, then we will sort by file size
            //Smaller files get a higher priority
            if (self.fileSize > other.fileSize) {
                retVal = true;
            }
        }

        retVal
    }
}

fn main() {
    //let req_vec: ~[sched_msg] = ~[];
    let req_vec: ~priority_queue::PriorityQueue<sched_msg> = ~priority_queue::PriorityQueue::new();
    let shared_req_vec = arc::RWArc::new(req_vec);
    let add_vec = shared_req_vec.clone();
    let take_vec = shared_req_vec.clone();
    
    let (port, chan) = stream();
    let chan = SharedChan::new(chan);

    let safe_count: ~[uint] = ~[0];
    let shared_visit = arc::RWArc::new(safe_count);
    
    // dequeue file requests, and send responses.
    // FIFO
    do spawn {
        let (sm_port, sm_chan) = stream();
        
        // a task for sending responses.
        do spawn {
            loop {
                let mut tf: sched_msg = sm_port.recv(); // wait for the dequeued request to handle
                match io::read_whole_file(tf.filepath) { // killed if file size is larger than memory size.
                    Ok(file_data) => {
                        println(fmt!("begin serving file [%?]", tf.filepath));
                        
                        /*
                        let fileType = match tf.filepath.filetype() {
                            Some(s) => s,
                            None => &""
                        };

                        let httpHeader: ~str = match fileType {
                            ".html" | ".htm" | ".php" => ~"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n",
                            _ => ~"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream; charset=UTF-8\r\n\r\n"
                        };


                        tf.stream.write(httpHeader.as_bytes());
                        tf.stream.flush();
                        */

                        tf.stream.write(file_data);
                        println(fmt!("finish file [%?]", tf.filepath));
                    }
                    Err(err) => {
                        println("ERROR IN SEND");
                        println(err);
                    }
                }
            }
        }
        
        loop {
            port.recv(); // wait for arrving notification
            do take_vec.write |vec| {
                if ((*vec).len() > 0) {
                    
                    //Since we are using a priority queue, we will use pop()
                    let tf = (*vec).pop();
                    println(fmt!("shift from queue, size: %ud", (*vec).len()));
                    sm_chan.send(tf); // send the request to send-response-task to serve.
                }
            }
        }
    }

    let ip = match FromStr::from_str(IP) { Some(ip) => ip, 
                                           None => { println(fmt!("Error: Invalid IP address <%s>", IP));
                                                     return;},
                                         };
                                         
    let socket = net::tcp::TcpListener::bind(SocketAddr {ip: ip, port: PORT as u16});
    
    println(fmt!("Listening on %s:%d ...", ip.to_str(), PORT));
    let mut acceptor = socket.listen().unwrap();
    
    loop {
        let stream = acceptor.accept();
        let stream = Cell::new(stream);
        
        // Start a new task to handle the each connection
        let child_chan = chan.clone();
        let child_add_vec = add_vec.clone();
        let child_arc = shared_visit.clone();
        do spawn {

            let mut actual_count:uint = 0;

            //We will use the cloned RWARC
            do child_arc.write |count_vec| {

                let prev_count = (*count_vec).pop();
                let new_count: uint = prev_count + 1;

                actual_count = new_count;

                (*count_vec).push(new_count);
            }
            
            let stream = stream.take();

            match stream {
                Some(s) => {

                    let mut stream = s;

                    let mut buf = [0, ..500];
                    stream.read(buf);
                    let request_str = str::from_utf8(buf);
                    
                    let req_group : ~[&str]= request_str.splitn_iter(' ', 3).collect();
                    if req_group.len() > 2 {
                        let path = req_group[1];
                        println(fmt!("Request for path: \n%?", path));
                        
                        let file_path = ~os::getcwd().push(path.replace("/../", ""));
                        if !os::path_exists(file_path) || os::path_is_dir(file_path) {
                            println(fmt!("Request received:\n%s", request_str));
                            let response: ~str = fmt!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
                                 <doctype !html><html><head><title>Hello, Rust!</title>
                                 <style>body { background-color: #111; color: #FFEEAA }
                                        h1 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red}
                                        h2 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green}
                                 </style></head>
                                 <body>
                                 <h1>Greetings, Krusty!</h1>
                                 <h2>Visitor count: %u</h2>
                                 </body></html>\r\n", actual_count);

                            stream.write(response.as_bytes());
                        }
                        else {
                            // Requests scheduling

                            let mut streamPriority: int = 0;

                            //Retrieving the requesting IP address
                            let ipStr: ~str = match (stream).peer_name() {
                                Some(pr) => pr.ip.to_str(),  
                                None => ~"0.0.0.0"
                            };

                            //Split the IP address by '.' so that we can compare
                            let ipSplit: ~[~str] = ipStr.split_iter('.').filter(|&x| x != "")
                                 .map(|x| x.to_owned()).collect();

                            //Assign priority based on geography or if localhost
                            if ( (ipSplit[0] == ~"127" && ipSplit[1] == ~"0") || (ipSplit[0] == ~"128" && ipSplit[1] == ~"143")
                                || (ipSplit[0] == ~"137" && ipSplit[1] == ~"54") ) {

                                streamPriority = 1;
                            }

                            
                            //Get filetype in order to determine HTTP header type
                            let fileType = match file_path.filetype() {
                                Some(s) => s,
                                None => &""
                            };

                            //In order to optimize for the benchmark, we will send the HTTP header quickly before adding to the queue
                            let httpHeader: ~str = match fileType {
                                ".html" | ".htm" | ".php" => ~"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n",
                                _ => ~"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream; charset=UTF-8\r\n\r\n"
                            };

                            stream.write(httpHeader.as_bytes());
                            stream.flush();


                            //Retrieve file info for additional latency fixes
                            let fileInfo = match std::rt::io::file::stat(file_path) {
                                Some(s) => s,
                                None => fail!("Could not access file stats")
                            };

                            let msg: sched_msg = sched_msg{stream: Some(stream), filepath: file_path.clone(), topPriority: streamPriority, fileSize: fileInfo.size};
                            let (sm_port, sm_chan) = std::comm::stream();
                            sm_chan.send(msg);
                            
                            do child_add_vec.write |vec| {
                                let msg = sm_port.recv();
                                (*vec).push(msg); // enqueue new request.
                                println("add to queue");
                            }
                            child_chan.send(""); //notify the new arriving request.
                            println(fmt!("get file request: %?", file_path));
                        }
                    }
                    println!("connection terminates")

                },
                None => ()
            }

            
        }
    }
}