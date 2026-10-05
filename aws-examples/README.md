<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# AWS examples

Rust programs that run on Amazon Web Services. These are **examples**: each one stands on its own and can be deployed to your own AWS account.

## AWS Lambda

| Example | What it is | Concepts demonstrated |
|---|---|---|
| [Hello world](aws-lambda/hello-world/) | the smallest useful Lambda function: an HTTP greeting | what Lambda and "serverless" are, why Rust fits, and why user input must never be served as HTML (a real XSS fix) |
| [User API with DynamoDB](aws-lambda/user-api-dynamodb/) | a complete user service: registration, login, JWT access tokens, refresh token rotation, three DynamoDB tables, a SAM template | a realistic serverless backend, integration tests against DynamoDB Local, and a **security review** with a test for every attack it fixes |

Both examples were reviewed and modernised. Each README ends with a table of what was found and fixed, including an account-takeover hole, a reflected XSS, and tests that had been passing without testing anything. They're good examples of why the [Software engineering with AI](../software-engineering-with-ai/) course insists on reviewing code, and on tests that really run.

## What you need

- [Rust](https://www.rust-lang.org/tools/install), for building and testing locally;
- [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html), for running Lambda functions locally and deploying them;
- an AWS account, only for deploying;
- for the user API's integration tests, DynamoDB Local (see its README).
