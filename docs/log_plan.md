I'm using opentelemetry to export observability data. Logs are done in rust using a combination of the log crate and opentelemetry's crates.

# Understanding Core Crates
`opentelemetry` - the opentelemetry api.
`opentelemetry_sdk` - the opentelemetry sdk.
`opentelemetry-stdout` - An exporter that writes telemetry data to standard out(aka the terminal). Is not for production use.

Log:
- when the server is listening and at what address
- context about the request recieved (the sender, and the full request)
- the response sent
- Shutdown

We can optimized log sizes by using links to static files instead of the full file contents.
