# Simple Rust HTTP Server

Created for educational purposes only. This is a rust http file server for vite generated static files.

**Topics Learned:**
- Rust
- Linux
- [Concurrency](/docs/concurrency.md)
	- Hopefully: Atomics
- OpenTelemetry Instrumentation

## Replication

The frontend is generated via this script
```shell
deno run -RWE npm:create-vite-extra@latest
```

With these configurations:
```shell
# relevant configuration
✔ Select a template: › deno-react
✔ Select a variant: › TypeScript
```

Backend uses the standard `cargo init` command.

## Topics:
### Linux
#### System Calls Used:
[poll(2)](https://www.man7.org/linux/man-pages/man2/poll.2.html)
[bind(2)](https://man7.org/linux/man-pages/man2/bind.2.html)
[listen(2)](https://man7.org/linux/man-pages/man2/listen.2.html)
[socket(2)](https://man7.org/linux/man-pages/man2/socket.2.html)
[sigaction(2)](https://man7.org/linux/man-pages/man2/sigaction.2.html)
[accept(2)](https://man7.org/linux/man-pages/man2/accept.2.html)
[getpeername(2)](https://www.man7.org/linux/man-pages/man2/getpeername.2.html)
[recv(2)](https://man7.org/linux/man-pages/man2/recv.2.html)
[send(2)](https://man7.org/linux/man-pages/man2/send.2.html)
[errno(3)](https://www.man7.org/linux/man-pages/man3/errno.3.html)

alternatively we could have used io_uring([liburing](https://github.com/axboe/liburing), [examples](https://unixism.net/loti/index.html#), [white paper](https://kernel.dk/io_uring.pdf)).
- https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io
#### Linux systems Used:
Socket Management
File Management
Linux Networking
Linux Signals
Linux Process
Linux Threads
Linux Polling
Running Programs in Linux
Linux give files and sockets file descriptors:
- How are these managed
How are Linux processes managed?
What are Linux processes
- Virtual address spaces
- Linux tasks (kernel or user space)
- How does the kernel manage thread
How does the Linux file abstraction work
How does the linux file system work
How to configure processes from clone(2) and fork(2)?
[Linux kernel scheduler](http://www.ibm.com/developerworks/library/l-completely-fair-scheduler/)
users and permissions??
Linux [capabilities(7)](https://man7.org/linux/man-pages/man7/capabilities.7.html), [credentials(7)](https://man7.org/linux/man-pages/man7/credentials.7.html) . https://stackoverflow.com/questions/8463741/how-linux-handles-threads-and-process-scheduling
### Rust
Iteration
Abstraction
Ownership
- Automatic Drops

Defensive programming.

Stack and Heap
Automatic Drops
Channels

alternative async
### Opentelemetry
Logs,
Metrics,
Trace
