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

We also determine if a file is in the cache. Files in the cache are given a higher priority. Thus the final priority ordering is as follows:

1. Geographic Location (to satisfy Wahoo-First)
2. Presence in the cache
3. File size (smaller = better)

If there is a tie for one criteria, the next criteria is used as a tiebreaker.

Problem 4
=========

We have made several design decisions in order to incorporate server-side includes:

+ Only html files (.html, .htm) are checked for SSI. This cuts down on overhead, especially when serving large binary files
+ Files with server-side includes are never added to the cache. This is because the wait for the process to execute defeats the purpose of the cache.
+ We have a modified version of gash that reads the command to execute from the program arguments rather than the standard input. We modified Makefile so that it is compiled along with zhtta.rs. 

If an html file contains an SSI, the command is stripped in order to determine the program to run, as well as its command-line arguments. The output of the program then replaces the entire SSI. 

Problem 5
=========

Need to address issues involving httperf segfault on Mac OS. 

Problem 6
=========

Caching involves many design decisions and tradeoffs. The overview would be as follows:

A list (stored as an array) keeps a record of all the files requested. The elements in the list are a struct that contains:

```
struct cache_item {
    name: ~str,
    in_use_flag: bool,
    data: ~[u8],
    count: uint,
    size: u64,
    modified: u64
}
```

Regardless of whether or not the item is in the cache, the item exists in the list since we need its historic request count in order to cache the most popular items. If it is not in the cache, we set `in_use_flag = false` and `data = ~[]` in order to save space. We are using the historic count, which might not be ideal since it does not take into account the recentness of the request. A solution to this would be to have a function that decays the file's count with time so that recent requests have a higher priority. 

The cache manager sorts the items in the cache list by request count. The performance overhead is minimal since we are using a sorting algorithm that sorts in place (thus not consuming additional memory). 

For each item in this ordered list, it checks to see if the file is currently in the cache. If it is not in the cache, a number of checks are made to make sure that it is a valid candidate for the cache, and adds it if it passes the criteria (for example, we do not add files with Server-Side Includes since this defeats the purpose of the cache). 

If the file is already in the cache (`in_use_flag = true`) and there is space remaining, the cache manager will check to see if the file was updated since being added to the cache. If it has, it will update the data and time modified values so that the cache remains up-to-date. 

The task running the cache manager pauses between loops (after it releases the read/write lock) so that we do not experience resource starvation. 

Problem 7
=========

+ Since httperf measures the response rate by the first byte received, the HTTP header is sent before sending the request to the request manager. 
+ The cache size and refresh rate can be set from an optional configuration file. If the file does not exist, default values are used. 
+ A more thorough check to the client request is made to make sure that the request doesn't attempt to go to the server's parent directory.
+ A special version of gash has been added that reads in the instructions from the program arguments to support server-side gashing.