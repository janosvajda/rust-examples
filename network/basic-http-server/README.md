# Basic Rust HTTP Server

A simple example of a basic HTTP server written in Rust. Demonstrates handling GET requests with a specific parameter and responding accordingly. The example does not use any external library.

## Functionality

The server listens on port 3000. It responds to GET requests with parameter `hello` by sending the message "world". For requests without or with a different parameter, it returns a 404 Not Found error message.

## Testing

To test using a web browser:

1. Run the server:

cargo run


2. Open your web browser and visit:

http://127.0.0.1:3000/?hello

Expected response: "world"

3. Test the 404 response by visiting:

http://127.0.0.1:3000/?other

Expected response: "404 Not Found"

To test using `curl` in your terminal:

```sh
curl http://127.0.0.1:3000/?hello
```


You can run test:

cargo test