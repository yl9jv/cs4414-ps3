ps3
===


Problem 1
=========

A safe visitor count was implemented using `extra::arc::RWArc`. The implementation may be a little inefficient since we are storing the value in a vector (due to pointer complications encountered otherwise) in the following manner:

```
let mut actual_count:uint = 0;

            //We will use the cloned RWARC
            do child_arc.write |count_vec| {

                let prev_count = (*count_vec).pop();
                let new_count: uint = prev_count + 1;

                actual_count = new_count;

                (*count_vec).push(new_count);
            }
```

Since the code is contained within the spawn, `actual_count` retains the value of the visitor count at that instance. By using this approach, we were able to avoid `unsafe` blocks and the use of static variables. 

Problem 2
=========

In order to implement the WahooFirst strategy, we modified several things. First, IP addresses from Charlottesville (plus the local machine address) are given a priority of 1 (0 otherwise). 

We replaced the received message vector with a priority queue and a comparator for `sched_msg` as well as added a priority field to the struct. The priority queue is ordered by IP priority, thus allowing us to meet the goals. 

Problem 3
=========

When a request is made, we access the file size via `rt::io::file::stat(file_path)`. We modified the `sched_msg` struct to include a field for file size. We then changed its comparator so that it first orders by IP address priority in order to satisfy Problem 2, but then modified it so that ties are ranked by file size. Smaller files (thus quicker to serve) are given a higher priority. 

Problem 4
=========

Problem 5
=========

Need to address issues involving httperf segfault on Mac OS. 

Problem 6
=========

Implemented. Need to elaborate. 

Problem 7
=========

+ Send header as soon as request is received. Currently gets file type by going to the HDD, perhaps can be modified so that it slices at the last '.' to avoid I/O overhead
+ Idea: Separate queue for cache and non-cache items
+ Idea: Optional command line argument to pass in cache size, cache refresh rate
+ Idea: Have some server parameters set by a config file, which can then be modified by issuing a special request via HTTP (remotely editing the config file, perhaps by creating a simple gash script)