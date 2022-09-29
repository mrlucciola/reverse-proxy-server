Start the origin server with `cargo run --bin origin`

Start the reverse-proxy server with `cargo run --bin proxy`

Make requests using command `curl "localhost:8081" -d "https://blockstream.info/api/blocks/0" -X GET`

## TODOs

- Validation for request body (url) on proxy (and possibly origin)
- More thorough validation for incoming HTTP requests (specific domains and endpoints)
- Handle all response error types (propagate to client HTTP response - appropriate HTTP status codes and messages)
- Clean up unused error types
- Refactor code to be more modular
- Refactor code to be more readable
- Add descriptions to all structs, functions and methods
- Restrict requests to specific sites and endpoints, create a payload body struct for each supported API call

## TTL implementation

### Simple/First pass:

Set cache size limit - to _x_ # of entries

After client response is sent:

- Iterate through all entries in the cache HashMap,
- Read timestamp value on the request (each entry is a request, key is URL)
- If greater than 30 seconds, delete entry from cache
- If at cache limit, remove oldest entry, (and insert current response in its place)

Check timestamp on the response stored in cache
