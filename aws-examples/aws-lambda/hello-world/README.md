<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Hello world on AWS Lambda

The smallest useful AWS Lambda function in Rust: it answers an HTTP request with a greeting. Call it with `?name=Ana` and it answers `Hello Ana, this is an AWS Lambda HTTP request`.

## What AWS Lambda is

**AWS Lambda** runs your code without a server of your own: you upload a program, and AWS starts it when a request arrives. You pay per request and per millisecond of running time, and nothing while nobody calls it. That's why it's called **serverless**: there are still servers, but managing them is Amazon's job.

A request reaches your code like this:

```text
browser ──HTTP──► API Gateway (or a Lambda function URL) ──event──► your Lambda ──► response
```

**Why Rust is a good fit for Lambda:**
- **Fast cold starts:** when no instance is warm, AWS must start your program first, and a small native Rust binary starts in milliseconds.
- **Little memory:** Lambda bills by memory size, and Rust needs little.
- **No runtime to install:** the program is one compiled binary.

## The code

```rust
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    run(service_fn(function_handler)).await          // wait for requests, call the handler for each
}
```

```rust
pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let who = event.query_string_parameters_ref().and_then(|p| p.first("name")).unwrap_or("world");
    let who: String = who.chars().take(100).collect();
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain; charset=utf-8")
        .body(format!("Hello {who}, this is an AWS Lambda HTTP request").into())?)
}
```

The `lambda_http` crate turns the Lambda event into an ordinary HTTP `Request`, so the handler looks like any web handler. Because the handler is a plain async function, it's tested by calling it directly, with no AWS involved.

## Never serve user input as HTML

The first version of this example answered with `content-type: text/html`, with the `name` from the URL inside. So this link:

```text
https://…/?name=<script>stealCookies()</script>
```

would make the visitor's browser **run that script**, on your site, with access to their session. This is called **reflected cross-site scripting (XSS)**, one of the most common web vulnerabilities, and it was in a "hello world".

The fix is to answer with `text/plain`: the browser then shows the characters `<script>` instead of running them. The name is also cut to 100 characters, since nobody's name is longer. If a response really must be HTML, every piece of user input in it must be **escaped**: `<` becomes `&lt;`, and so on. A template engine does this automatically.

The test `html_in_the_name_is_never_served_as_html` keeps it fixed.

## Run and test it

You need [Rust](https://www.rust-lang.org/tools/install), and [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html) for running it locally and deploying it.

```bash
cargo test                       # unit tests: no AWS needed
cargo lambda watch               # terminal 1: a local Lambda emulator
cargo lambda invoke --data-example apigw-request     # terminal 2: send a sample request
```

```text
{"statusCode":200,"headers":{},"multiValueHeaders":{"content-type":["text/plain; charset=utf-8"]},"body":"Hello me, this is an AWS Lambda HTTP request","isBase64Encoded":false}
```

To deploy it to your AWS account: `cargo lambda build --release`, then `cargo lambda deploy`.

## What was fixed

| Problem | Fix |
|---|---|
| reflected XSS: user input in a `text/html` response | `text/plain`, and names cut to 100 characters |
| **the tests never ran**: `test = false` in `Cargo.toml` turned off the binary's unit tests, so `cargo test` ran nothing | removed; four tests now run |
| `lambda_http` 0.13, edition 2021 | `lambda_http` 0.17 (the same version as the other example), edition 2024 |

Back to [AWS examples](../../)
