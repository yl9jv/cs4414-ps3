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
// Version 0.1

extern mod extra;

use std::rt::io::*;
use std::rt::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::println;
use std::cell::Cell;
use std::{os, str, io};
use extra::arc;
use std::comm::*;
use extra::priority_queue;

static PORT:    int = 4414;
static IPV4_LOOPBACK: &'static str = "127.0.0.1";
static mut visitor_count: uint = 0;

struct sched_msg {
    stream: Option<std::rt::io::net::tcp::TcpStream>,
    filepath: ~std::path::PosixPath,
    topPriority: int,
}

impl sched_msg {

    //fn dispIP(&self) {

        //let mut xx =  self.stream.clone();
        /*
        match self.stream {
            Some(st) => {
                match st.peer_name() {
                    Some(pr) => {println("works");},
                    None => ()
                }
            },
            None => ()
        }
        */
    //}
}

impl Ord for sched_msg {
    fn lt(&self, other: &sched_msg) -> bool {

        let mut retVal:bool = false;

        if (self.topPriority <= other.topPriority) {
            retVal = true;
        }
        retVal
    }
}

fn main() {

    let safe_count: ~[uint] = ~[0];
    let shared_visit = arc::RWArc::new(safe_count);

    //let req_vec: ~[sched_msg] = ~[];
    let req_vec: ~priority_queue::PriorityQueue<sched_msg> = ~priority_queue::PriorityQueue::new();

    let shared_req_vec = arc::RWArc::new(req_vec);
    let add_vec = shared_req_vec.clone();
    let take_vec = shared_req_vec.clone();
    
    let (port, chan) = stream();
    let chan = SharedChan::new(chan);


    
    
    // add file requests into queue.
    do spawn {
        while(true) {
            do add_vec.write |vec| {
                let tf:sched_msg = port.recv();
                (*vec).push(tf);
                println("add to queue");
            }
        }
    }
    
    // take file requests from queue, and send a response.
    do spawn {
        while(true) {
            do take_vec.write |vec| {
                let mut tf = (*vec).pop();
                
                match io::read_whole_file(tf.filepath) {
                    Ok(file_data) => {

                        if(file_data.len() > 1) {
                            let tfLeft = file_data.slice_to(1).to_owned();
                            let tfRight = file_data.slice_from(1).to_owned();

                            println!("Writing left: {:?}", tfLeft);
                            tf.stream.write(tfLeft);
                            //tf.stream.flush();
                            println!("Writing right: {:?}", tfRight);
                            tf.stream.write(tfRight);
                        }
                        else {
                            tf.stream.write(file_data);
                        }
                    }
                    Err(err) => {
                        println(err);
                    }
                }
            }
        }
    }
    
    let socket = net::tcp::TcpListener::bind(SocketAddr {ip: Ipv4Addr(127,0,0,1), port: PORT as u16});
    
    println(fmt!("Listening on tcp port %d ...", PORT));

    let mut acceptor = socket.listen().unwrap();
    
    // we can limit the incoming connection count.
    //for stream in acceptor.incoming().take(10 as uint) {

    //Note: acceptor.incoming() is blocking
    loop {


        let stream = acceptor.accept();
        let stream = Cell::new(stream);
        

        // Start a new task to handle the connection
        let child_chan = chan.clone();

        let child_arc = shared_visit.clone();

        do spawn {

            //This value will be assigned the actual count for a particular user's specific request
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

                        //if path in cache -> serve file, else, proceed below

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
                            
                            let mut streamPriority: int = 0;


                            let ipStr: ~str = match (stream).peer_name() {
                                Some(pr) => pr.ip.to_str(),  
                                None => ~"0.0.0.0"
                            };

                            let ipSplit: ~[~str] = ipStr.split_iter('.').filter(|&x| x != "")
                                 .map(|x| x.to_owned()).collect();

                            if ( (ipSplit[0] == ~"127" && ipSplit[1] == ~"0") || (ipSplit[0] == ~"128" && ipSplit[1] == ~"143")
                                || (ipSplit[0] == ~"137" && ipSplit[1] == ~"54") ) {

                                streamPriority = 1;
                            }

                            //println!("IP ADDRESS {:?} PRIORITY {:?}", ipSplit, streamPriority);
                            
                            
                            
                            match io::file_reader(file_path) {
                                Ok(the_reader) => {

                                }
                            }


                            let msg: sched_msg = sched_msg{stream: Some(stream), filepath: file_path.clone(), topPriority: streamPriority};
                            child_chan.send(msg);
                            
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
