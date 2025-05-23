# Multi-threading
## Notes
- **Concurrency & Parallelism** - Rust threads are OSThreads, depending on hardware and operating system they can be concurrent or/and parallel.
- **Signals: Main vs Child Threads** - relevant to this program, but by default only the main thread of a rust program receives signals.
	- **Potential Solution:** Creating the threads with [clone(2)](https://www.man7.org/linux/man-pages/man2/clone.2.html) could likily solve this problem.
	- **Potential Solution:** A custom channel designed to send the messages to in order to all receivers could solve this problem.
- **Threads** - Linux threads are child processes with specific configuration. Rust exposes process creation in it's standard library thread and process abstractions. I chose to use Rust's thread abstraction.
- **Inter Thread Communication:** Shared state and message passing where used for communication between threads.

## Plan:
**Main Thread:**
- initializes telemetry systems
- initializes signal handling
- receives signals
- accepts connections and passes them to request handling child threads via channel.
- Handles graceful shutdowns
	- joins child threads
	- exports remaining telemetry data
	- shutdowns telemetry systems

**Child Threads:**
- Receives requests from main thread via channel.
- Build Responses
- Send response
