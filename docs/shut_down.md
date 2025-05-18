# Desired Behavior

On first kill signal **schedule shutdown**, on second kill signal **force shutdown**.
- **Scheduled shutdown** should allow threads to finish serving their current requests then terminate, threads not serving requests should terminate immediately. If the shutdown takes to long (*to be specified*) the server should force shutdown.
- **Force shutdown** should immediately export all telemetry data, and then terminate program immediately, operating system is expected to do the dirty work(*to be understood*).

## Problems (05/11/2025)
- upon recieving the first signal the system does not shut down until all threads have processed a request. This can be indefinite if as the syscalls can block indefinitely.
  - a core reason this happen is that i didn't specify the behavior fully a head of time.
  - the force shutdown works as indended, but could use better shutdown and telemetry exporting logic.

### Possible Solutions
1. can we set threads to recieve the signal? So the syscalls interrupt
2. change the default behaviour, so that the threads end early
  - set socket to nonblocking
  - put a timeout on calls and check for the signal


Change the flag to a atomic boolean.
use massage passing and job queue channel to pass requests to worker threads.
- use channel shutdown to to indicate graceful shutdown (not sure how)

add more abstraction to server?
- put all telemetry related operations into abstracting functions
  - init_telemetry, shutdown_telemetry, force_export_telemetry
  - set global telemetry state
  - only send useful telemetry data
- put all values into a application context struct and pass it around
  - struct ApplicationContext {telemetry: Telemetry,...}

we can move accepting requests to the main thread, and pass them via channels.
we can communicate terminatation via signals but that would require the creatation of a new channel type.

init_server(socket_addr) -> server_context
begin_worker(worker_count, server_context, request_receiver, request_handler)
accept_requests(request_sender, server_context)


### Writing `shutdown_telemetry` & `force_export_telemetry`
#### shutdown_telemetry
**Problem**
- I don't know if all the shutdown functions export the remaining telemetry data.
- I don't know how to handle the various error cases (there is no explicit guidance on handling the various error cases)
#### force_export_telemetry

abstract the server into a server struct with methods:
- `init_server`
- `begin_request_handlers`
- `accept_requests`
- `wait_request_handlers_shutdown`
