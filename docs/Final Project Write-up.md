# Final Project Write-up

These Headings are steps in the process.

# Single Request Handling

Requirements:

- TCP - HTTP - WebSocket Server
- Sending: Files, JSON

Steps:

- Set up

Learning:

- What is & how does TCP/ip work?
    - Difference between IPv4 and IPv6
- What does RAW TCP response look like?
- The [HTTP](https://datatracker.ietf.org/doc/html/rfc9112) and [WebSocket](https://datatracker.ietf.org/doc/html/rfc6455) Protocols
    - [WebSocket Browser API](https://websockets.spec.whatwg.org/#the-websocket-interface)
    - [MDN HTTP Resources](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Resources_and_specifications)
- Linux and TCP?
    - Whats the history here?
    - When did TCP come out and why?
    - When did linux come out and why?
    - How does the os handle networking
    - All linux systems use sockets as a abstraction over networking.
        - This abstraction connects networking to the linux file system.
        - Linux uses files an a abstraction over a ton of stuff.
    - Manual Pages used
        - [socket(2)](https://man7.org/linux/man-pages/man2/socket.2.html) - create an endpoint for communication
            - [getaddrinfo(3)](https://man7.org/linux/man-pages/man3/getaddrinfo.3.html) - example usage of a socket
            - [close(2)](https://man7.org/linux/man-pages/man2/close.2.html) - used to close sockets
            - [listen(2)](https://man7.org/linux/man-pages/man2/listen.2.html) - listen for connections on a socket
            - [write(2)](https://man7.org/linux/man-pages/man2/write.2.html)/[send(2)](https://man7.org/linux/man-pages/man2/send.2.html) - send data over the sockets
        - [bind(2)](https://man7.org/linux/man-pages/man2/bind.2.html) - bind a name to a socket
        - [ip(7)](https://man7.org/linux/man-pages/man7/ip.7.html) - linux IPv4 protocol implementation
        - [socket(7)](https://man7.org/linux/man-pages/man7/socket.7.html) - Linux socket interface
        - [htonl(3p)](https://man7.org/linux/man-pages/man3/htons.3p.html) - convert an number into a port value
        - [inet_addr(3p)](https://man7.org/linux/man-pages/man3/inet_addr.3p.html) - This function shall convert the string pointed to by
        cp, in the standard IPv4 dotted decimal notation, to an integer
        value suitable for use as an Internet address
        - [sockaddr(3type)](https://man7.org/linux/man-pages/man3/sockaddr.3type.html) - [struct sockaddr_in](https://www.gta.ufrj.br/ensino/eel878/sockets/sockaddr_inman.html) - structures for handling internet addresses
        - [memset(3)](https://man7.org/linux/man-pages/man3/memset.3.html) - fill memory with a constant byte
        - [tcp(7)](https://man7.org/linux/man-pages/man7/tcp.7.html) - TCP protocol
        - Password less web authentication system. Web sessions project

There is no C client for opentelemetry so I will not use one.

For the rust and maybe the golang implementations I’ll use one.

Sources:

- https://linux-kernel-labs.github.io/refs/heads/master/labs/networking.html
- https://www.beej.us/guide/bgnet/
- Issues
    - Getting the full request
    - Parse the request
    - routing the request
    - Building the response

# Single Request Graceful Failure

Signal

- Address all proper kill signals
- Add File Serving
- Add Telemetry

# Multiple Request Handling

Can I write a rust program that put a single thread on my 10-core cpu.

1 parent controller thread

9 child processor thread

# Multiple Request Graceful Fail

