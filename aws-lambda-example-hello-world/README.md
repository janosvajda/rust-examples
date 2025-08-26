# Introduction

aws-lambda-example-hello-world is a very basic Rust project that implements an AWS Lambda function in Rust.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html)

## Testing

You can run regular Rust unit tests with `cargo test`.

If you want to run integration tests locally, you can use the `cargo lambda watch` and `cargo lambda invoke` commands to do it.

Firstly, you should run `cargo lambda watch` to start a local server on the first terminal. When you make changes to the code, the server will automatically restart.

After this, open a second terminal, and run:

```bash
cargo lambda invoke --data-example apigw-request
```

You should see this result:

```bash
{"statusCode":200,"headers":{},"multiValueHeaders":{"content-type":["text/html"]},"body":"Hello me, this is an AWS Lambda HTTP request","isBase64Encoded":false}
```

For generic events, where you define the event data structure, you can create a JSON file with the data you want to test with. For example:

```json
{
    "command": "test"
}
```

Then, run `cargo lambda invoke --data-file ./data.json` to invoke the function with the data in `data.json`.

For HTTP events, you can also call the function directly with cURL or any other HTTP client. For example:

```bash
curl https://localhost:9000
```

## Building

To build the project for production, run `cargo lambda build --release`. Remove the `--release` flag to build for development.

Read more about building your lambda function in [the Cargo Lambda documentation](https://www.cargo-lambda.info/commands/build.html).