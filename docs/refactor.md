
Refactor
- timeout syscalls such that
- accept requests in main thread, and pass them to worker threads via a channel.
- place server configuration in a static object.
- use atomic boolean for flags
