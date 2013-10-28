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

We have made several design decisions in order to incorporate server-side includes:

+ Only html files (.html, .htm) are checked for SSI. This cuts down on overhead, especially when serving large binary files
+ Files with server-side includes are never added to the cache. This is because the wait for the process to execute defeats the purpose of the cache.

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
    ssi_flag: bool,
    hash: ~str,
    data: ~[u8],
    count: uint,
    size: u64
}
```

Regardless of whether or not the item is in the cache, the item exists in the list since we need its historic request count in order to cache the most popular items. If it is not in the cache, we set `in_use_flag = false` and `data = ~[]` in order to save space.

There are two cache managers: The primary and secondary. 

Primary Cache Manager
---------------------
The primary sorts the list by item request count using quicksort (it sorts in place, thus avoiding memory overhead costs). For each item in the ordered list, it opens the file and checks to see if it is smaller than the cache size remaining. If it is, it then checks to make sure that it does not have server-side includes (if it is an HTML file). It then updates the item's setting by setting its `data` to the file data, `in_use_flag = true`, and `hash = md4::hash(data)`. It then subtracts the file size from the remaining cache space available. 

If `in_use_flag` was already true, then there is no need to update settings since the file is already in the cache. All it does in this case is subtract the file size from the cache space available. 

This process loops through all items in the list until it finishes or there is no more free space in the cache. 

In order to avoid resource starvation, this process loops every `CACHE_MANAGER_A_RATE` milliseconds. 

Secondary Cache Manager
-----------------------
The secondary checks to make sure that files in the cache are always up-to-date. 

For all the items in the cache list that have `in_use_flag = true`, its file is opened and its data is hashed using MD4. If the hashes differ, then the item's data is replaced, and its hash is set to the new hash. Otherwise, it goes on to the next one. 

In order to avoid resource starvation, this process loops every `CACHE_MANAGER_B_RATE` milliseconds, which can be a larger number, such as 6000 milliseconds. 

Problem 7
=========

+ Since httperf measures the response rate by the first byte received, the HTTP header is sent before sending the request to the request manager. 
+ The program can handle the security problem caused by dangerous request similar to "/./.././", which makes it possible to access any file on the host.

+ Idea: Separate queue for cache and non-cache items
+ Idea: Optional command line argument to pass in cache size, cache refresh rate
+ Idea: Have some server parameters set by a config file, which can then be modified by issuing a special request via HTTP (remotely editing the config file, perhaps by creating a simple gash script)
