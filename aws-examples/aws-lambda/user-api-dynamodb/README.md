<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# User API on AWS Lambda with DynamoDB

A user-service example in Rust, built to run on AWS Lambda: people can **register**, **log in**, read and update **their own** profile, and stay logged in with **tokens**. The data lives in DynamoDB.

> **No AWS account needed.** You can run every test, and try the API by hand, on your own computer, against DynamoDB Local. Deploying to AWS is optional and described at the very end.

## Contents

1. [What it is](#1-what-it-is)
2. [Run it locally](#2-run-it-locally)
3. [How it works](#3-how-it-works)
4. [The code](#4-the-code)
5. [Security](#5-security)
6. [Reference: API, data and configuration](#6-reference)
7. [Deploying to AWS (optional)](#7-deploying-to-aws-optional)
8. [Under the hood: many requests at once](#8-under-the-hood-many-requests-at-once)
9. [Moving existing data](#9-moving-existing-data)
10. [What the test results mean](#10-what-the-test-results-mean)

## 1. What it is

```text
                       ┌──────────────────────── AWS ─────────────────────────┐
 client ──HTTPS──► API Gateway ──► Lambda (this Rust program) ──► DynamoDB      │
 (app, curl)           │                  │                        • Users      │
                       │                  │                        • UserCredentials
                       │                  │                        • UserRefreshTokens
                       │                  └──► SSM Parameter Store (the JWT secret)
                       └──────────────────────────────────────────────────────┘
```

- **API Gateway** receives HTTP requests and hands each one to the Lambda function.
- **The Lambda function** is this Rust program. AWS manages its execution environments and invokes the handler for incoming requests. Environments can be reused; their lifetime is not controlled by this program.
- **DynamoDB** is AWS's key-value database. It holds three tables: the users, their password hashes, and their refresh tokens.
- **SSM Parameter Store** keeps the secret used to sign login tokens, outside the code.

| Request | What it does | Needs a token? |
|---|---|---|
| `POST /users` | register a new user | no |
| `POST /login` | check email and password, return tokens | no |
| `GET /users?userId=…` | read **your own** profile | yes: your access token |
| `POST /users` with `userId` | update **your own** profile | yes: your access token |
| `POST /token/refresh` | swap a refresh token for new tokens | the refresh token itself |
| `POST /token/revoke` | log out: cancel a refresh token | the refresh token itself |

**What you'll learn here:** a realistic serverless backend in Rust; storing passwords safely; how login tokens work; integration tests against a real database; and what a security review of such a service finds.

## 2. Run it locally

You need [Rust](https://www.rust-lang.org/tools/install), and **DynamoDB Local**: Amazon's free version of DynamoDB that runs on your computer.

### Step 1: start DynamoDB Local (terminal 1)

**With Docker**, after starting Docker Desktop:

```bash
docker run --rm -p 8000:8000 amazon/dynamodb-local
```

**Or with Java 17 or newer**, without Docker: download DynamoDB Local from the official AWS page, [Deploying DynamoDB locally on your computer](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DynamoDBLocal.DownloadingAndRunning.html), and follow its instructions to start it on port 8000 with `-inMemory`.

Either way, leave it running. It keeps everything in memory (`-inMemory`; the Docker image does the same), so nothing is saved to disk. Stop it with **Ctrl+C** when you're done.

### Step 2: run the tests (terminal 2)

```bash
REQUIRE_DYNAMODB=1 cargo test
```

Every test should pass, including the security and concurrency cases in `security_flow`.

**Why `REQUIRE_DYNAMODB=1`?** Without a database, the integration tests print `skipping integration test: DynamoDB not reachable` and pass, so a plain `cargo test` still works anywhere. But then "passed" would mean "tested nothing". With `REQUIRE_DYNAMODB=1`, a missing database is a **failure**, so a green result really means the tests ran. CI does exactly this: on every pull request and every push to `main`, it starts DynamoDB Local next to the tests.

### Step 3 (optional): try the API by hand

This runs the Lambda function itself on your computer. It needs [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html), and DynamoDB Local still running from step 1.

```bash
cargo lambda watch --env-file env/local.env          # terminal 2: the Lambda, on port 9000
```

`env/local.env` points the program at DynamoDB Local, creates the tables on startup, and sets a local JWT secret. `cargo lambda watch` reloads the program when you change the code. Then, in a third terminal:

```bash
# register
curl -X POST http://127.0.0.1:9000/users \
  -H 'content-type: application/json' \
  -d '{"userName":"alice","email":"alice@example.com","password":"a-long-password","familyId":"fam-1"}'

# log in: copy userId, accessToken and refreshToken from the answer
curl -X POST http://127.0.0.1:9000/login \
  -H 'content-type: application/json' \
  -d '{"email":"alice@example.com","password":"a-long-password"}'

# read your own profile, with your access token
curl "http://127.0.0.1:9000/users?userId=<userId>" \
  -H "Authorization: Bearer <accessToken>"

# get new tokens with the refresh token (the old refresh token stops working)
curl -X POST http://127.0.0.1:9000/token/refresh \
  -H 'content-type: application/json' \
  -d '{"refreshToken":"<refreshToken>"}'

# log out
curl -X POST http://127.0.0.1:9000/token/revoke \
  -H 'content-type: application/json' \
  -d '{"refreshToken":"<the newest refreshToken>"}'
```

Try the failures too: read the profile without the `Authorization` header (`401`), or log in with a wrong password (`401`).

## 3. How it works

### Registering: never store a password

The password itself is never stored. It's turned into a **hash** with **Argon2**, and only the hash is kept, in the `UserCredentials` table:

```rust
let salt = SaltString::generate(&mut OsRng);                 // random, different for every user
Argon2::default().hash_password(password.as_bytes(), &salt)  // slow on purpose
```

- A **hash** is a one-way function: easy to compute from the password, practically impossible to reverse. At login, the typed password is hashed again and compared with the stored hash.
- The **salt** is random data mixed in, different for every user. Two users with the same password get different hashes, so one cracked password doesn't reveal the others.
- Argon2 is **slow on purpose**, and needs a lot of memory. Its duration depends on the configured parameters and machine. But an attacker who steals the table and tries billions of guesses is slowed down enormously. Fast hashes like SHA-256 are the wrong tool for passwords.

### Logging in: two kinds of token

After a correct password, the user gets **two** tokens:

| | Access token | Refresh token |
|---|---|---|
| what it is | a **JWT**: signed data that says who you are | a user-id prefix plus a random UUID credential, stored in `UserRefreshTokens` |
| used for | every request that needs to know who you are | only to get new tokens |
| lives for | 15 minutes | 7 days |
| checked by | verifying the **signature**, without the database | looking it up **in the database** |
| can it be cancelled? | this service does not revoke individual JWTs; expiry or a signing-key change invalidates them | yes: delete it (`/token/revoke`) |

**A JWT** ("JSON Web Token") has three parts, separated by dots: a header, the **claims** (here: the user id `sub`, the family id `fid`, and the expiry time `exp`), and a **signature**. The signature is computed from the header, the claims and a **secret** that only the server knows (HS256: HMAC with SHA-256). Changing a single character of the claims makes the signature wrong. So the server can trust a JWT without asking the database, which makes it fast.

**Why two tokens?** That speed has a cost: this service does not check a JWT revocation list. Hence the split:
- The access token is short-lived. If it's stolen, it's useless after 15 minutes.
- The refresh token can be cancelled at any time, because it lives in the database. Checking it there is slower, but it's only needed every 15 minutes.

### Refreshing: rotation

`POST /token/refresh` **uses up** the refresh token: it conditionally replaces the user's stored credential and returns a new pair. This is called **rotation**. A stolen refresh token stops working as soon as the real user refreshes. The replacement requires the old credential to match and remain unexpired in the same write. Two requests racing with that credential cannot both succeed. Signing the access token happens before the write, so a signing failure does not consume the refresh credential.

Each user has at most one active refresh token: logging in again replaces it.

### Who may do what

| Check | Question | Fails with |
|---|---|---|
| **authentication** | *who are you?* Is there a valid access token? | `401 Unauthorized` |
| **authorization** | *may you do this?* Is it your own profile? | `403 Forbidden` |

Reading or updating a profile needs both: a valid token, **and** a `userId` that matches the token's `sub` claim.

## 4. The code

| File | What's in it |
|---|---|
| `src/main.rs` | startup: logging, configuration, the JWT secret from SSM, then the Lambda runtime loop |
| `src/handlers.rs` | one function per endpoint, plus `authenticate` (checks the token) and `validate` (checks the input) |
| `src/auth.rs` | Argon2 hashing, issuing and verifying JWTs, refresh token generation |
| `src/user.rs` | the user record, and converting it to and from DynamoDB items |
| `src/context.rs` | `AppContext`: the DynamoDB client, table names and secret, shared by all requests |
| `src/runtime_env.rs` | working out the environment (`Local`, `Prod`, …) and the table names |
| `src/bootstrap.rs` | creating the tables, for local runs and tests |
| `src/error.rs` | the application's error type |
| `tests/` | integration tests, one file per flow (below) |
| `template.yaml`, `samconfig.toml` | the AWS infrastructure, for deploying (section 7) |

**Startup runs once, requests run many times.** `main` builds the DynamoDB client and reads the secret **once**, when AWS starts the function. Then `lambda_http::run` calls the handler for every request, reusing the same `AppContext`. Doing the expensive setup per request would make every request slower.

**The tests**, in `tests/`, each create their own tables with random names, so they can run in parallel without sharing data:

| File | Covers |
|---|---|
| `user_flow.rs` | registration, update with your own token, duplicates, families, and that an update keeps `createdAt` |
| `login_flow.rs` | login success and failure, refresh token issuance |
| `auth_flow.rs` | JWT claims and signature |
| `refresh_flow.rs` | refresh token rotation and revocation |
| `security_flow.rs` | **attacks that must fail**: changing someone else's password, reading someone else's profile, weak or malformed input, using one refresh token twice at once |

## 5. Security

A security review of this example found real problems. Each fix comes with a test that performs the attack and checks that it now fails:

| Problem | Fix | Test |
|---|---|---|
| **Account takeover.** `POST /users` with a `userId` was an "update" that set a new password for any email, with no authentication | updates need the user's own access token, and DynamoDB checks that the email belongs to that user in the same write | `nobody_can_change_someone_elses_password` |
| **Issued tokens were never checked.** `GET /users` returned anyone's record, email included, to anyone | `GET /users` needs the user's own access token. Tokens are verified: signature, HS256 only, not expired | `reading_a_user_needs_that_users_token` |
| **Login timing revealed registered emails.** An unknown email answered instantly; a known one only after Argon2's work | an unknown email runs a dummy Argon2 check first, to reduce the timing difference; exact response-time equality is not guaranteed | (timing: not unit-testable reliably) |
| **A refresh token could be used twice** by two simultaneous requests | rotation conditionally replaces the one stored credential, so only one reuse can win | `a_refresh_token_works_only_once_even_at_the_same_moment` |
| **The JWT secret silently fell back** to an environment variable when SSM failed, in any environment | the fallback is allowed only in `Local` | (in `main.rs`) |
| **No input limits:** empty passwords accepted; huge ones made Argon2 do unbounded work | passwords 8–1024 bytes, emails checked, names limited | `weak_or_malformed_input_is_rejected` |
| **Personal data in logs:** every 4xx response body (some with emails) was logged | only the status code is logged | (in `json_response`) |
| **An update reset `createdAt`** | the original creation time is kept | `user_crud_and_constraints_flow` |

Before the fixes, those tests failed: they proved each attack worked. One existing test, `user_flow.rs`, even tested the unprotected update as a *feature*. A test suite can lock a security hole in place, if nobody asks whether the behaviour is right.

**Not addressed here, worth doing in production:**
- **Store refresh tokens hashed.** Today they're stored as-is, so anyone who can read the table can use them. Store a cryptographic hash of the random credential in the user's stable row and compare that hash during rotation. Passwords need a password-hashing algorithm; a high-entropy random token has different hashing requirements.
- **Rate limiting** on `/login` and `/users`, against password guessing and mass registration. API Gateway usage plans or AWS WAF can do this in front of the Lambda.
- **Registration reveals existing emails** (`409 Conflict`). That's a common trade-off for a clear user experience. The alternative is to always answer "check your inbox" and send the details by email.
- **Keep the deployment private:** API Gateway with IAM authorisation or a private integration, so only your own services can call it.

## 6. Reference

### The API

| Request | Body or parameters | Answers |
|---|---|---|
| `POST /users` (register) | `{"userName", "email", "password", "familyId"}` | `201` and the user record; `400` invalid input; `409` email or name taken |
| `POST /users` (update) | the same, plus `"userId"`; header `Authorization: Bearer <accessToken>` | `201`; `401` no or invalid token; `403` not your user; `404` no such user |
| `GET /users` | `?userId=…`; header `Authorization: Bearer <accessToken>` | `200` and the user record; `401`; `403`; `404` |
| `POST /login` | `{"email", "password"}` | `200` with `accessToken`, `tokenType`, `expiresIn`, `refreshToken`, `refreshExpiresIn`, `userId`, `familyId`; `401` wrong email or password |
| `POST /token/refresh` | `{"refreshToken"}` | `200` with a new token pair; `401` invalid, used or expired |
| `POST /token/revoke` | `{"refreshToken"}` | `200` `{"revoked": true}` |

Rules:
- Passwords must be 8 to 1024 bytes long, and emails must look like `name@example.com`.
- Emails are unique across the system, and `familyId` + `userName` pairs are unique.
- Access tokens last 15 minutes and refresh tokens 7 days.
- Responses are JSON and include the user record. Passwords and their hashes are never returned.

### The DynamoDB tables

Each environment has its own three tables, named after it: `Users_Prod`, `Users_Local`, and so on.

**`Users_<env>`**: the user records.

| Attribute | Type | Notes |
|---|---|---|
| `userId` | string | partition key; generated when absent |
| `userName` | string | required; unique per family through transactionally written reservation rows; the GSI supports queries |
| `email` | string | required; indexed via `EmailIndex` |
| `familyId` | string | required grouping id; indexed via `FamilyIdIndex` |
| `createdAt` | string | RFC 3339 timestamp |
| `updatedAt` | string | RFC 3339 timestamp |

**`UserCredentials_<env>`**: `email` (partition key), `userId`, `familyId`, `passwordHash`.

**`UserRefreshTokens_<env>`**: `refreshToken` (partition key containing `ACTIVE#<userId>`), `token` (the current credential), `userId`, `familyId`, `expiresAt`. There is one active row per user; rotation replaces it in place.

To add an ordinary stored field, update `UserRecord` and its item conversion. DynamoDB attribute definitions in the template describe table/index keys, not every stored field; change the template when the key/index schema changes.

### Configuration

| Variable | Meaning |
|---|---|
| `ENVIRONMENT_NAME` | `Local`, `Prod`, `Staging`, …: decides the table names. Without it: `Local` under local tooling such as `cargo lambda watch`, otherwise `Prod` |
| `CREDENTIALS_TABLE_NAME`, `REFRESH_TOKEN_TABLE_NAME` | the two token-related tables |
| `JWT_SECRET_PARAMETER` | the SSM parameter that holds the JWT secret |
| `JWT_SECRET` | **local only**: used when the SSM lookup fails in the `Local` environment. In any other environment, a failed lookup stops the function, because signing tokens with some other secret would be far worse than not starting |
| `BOOTSTRAP_DYNAMODB_TABLES` | create the tables at startup: for local runs; deployed stacks leave it unset, because CloudFormation owns the tables |
| `DYNAMODB_ENDPOINT`, `AWS_ALLOW_HTTP` | point the SDK at DynamoDB Local over plain HTTP |

`env/local.env` sets all of these for local runs.

## 7. Deploying to AWS (optional)

Everything above works without AWS. This section is for anyone with an AWS account who wants to run the service for real. It needs the [AWS CLI](https://aws.amazon.com/cli/), the [SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/install-sam-cli.html) and Cargo Lambda.

### Build

```bash
cargo lambda build --release
```

The binary ends up in `target/lambda/aws-lambda-example-db/`. The stack uses the `provided.al2023` runtime, and `cargo lambda build` produces a compatible binary on macOS or Linux, with no Docker or cross-compilation setup.

### Check which account you'll deploy to

The AWS tools use the standard credential chain: a `--profile` argument first, then the `AWS_PROFILE` variable, then access keys in the environment, then your default profile. Check it before deploying:

```bash
aws sts get-caller-identity [--profile your-profile]
```

### Create the JWT secret

The stack reads the signing secret from SSM Parameter Store, at `/apps/aws-lambda-example-db/<environment>/JWT_SECRET` by default. The prefix is set by `JwtSecretParameterPrefix`; SSM forbids prefixes starting with `aws` or `ssm`. Create it once per environment:

```bash
aws ssm put-parameter \
  --name /apps/aws-lambda-example-db/Prod/JWT_SECRET \
  --type SecureString \
  --value 'replace-with-a-long-random-secret' \
  --overwrite
```

### Deploy

```bash
cargo lambda build --release
sam deploy --config-env prod --no-confirm-changeset
```

`samconfig.toml` supplies the stack name, the S3 bucket prefix and the capabilities. One command deploys the whole stack: the Lambda function, the API Gateway routes, the three DynamoDB tables and the IAM permissions. All three tables use `DeletionPolicy: Retain`, so deleting the stack leaves the data behind. Delete the tables by hand if you really want them gone.

### Other environments

The stack defaults to `EnvironmentName=Prod`. For another environment, create its secret first, then:

```bash
sam deploy --config-env prod --no-confirm-changeset --parameter-overrides EnvironmentName=Staging
```

| Environment | How to run or deploy | Users table |
|---|---|---|
| `Prod` | `sam deploy --config-env prod --no-confirm-changeset` | `Users_Prod` |
| `Staging` | the same, with `--parameter-overrides EnvironmentName=Staging` | `Users_Staging` |
| any name | the same, with `EnvironmentName=<name>` | `Users_<name>` |
| `Local` | `cargo lambda watch --env-file env/local.env` | `Users_Local` |

### Check the deployment

Find the API's address:

```bash
aws cloudformation describe-stacks \
  --stack-name aws-lambda-example-db-prod \
  --query 'Stacks[0].Outputs[?OutputKey==`UsersApiUrl`].OutputValue' \
  --output text
```

It looks like `https://abc123.execute-api.us-east-1.amazonaws.com/Prod/users`. Use it in place of `http://127.0.0.1:9000/users` in the `curl` commands from [section 2](#step-3-optional-try-the-api-by-hand).

The template sets `AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH=true`, which tells `lambda_http` to remove the stage (`/Prod`) from the path before routing. So the address can be used exactly as shown.

To look inside: `aws dynamodb scan --table-name Users_Prod` shows the records. The function's logs are in CloudWatch, under `/aws/lambda/aws-lambda-example-db`. CloudWatch also gets a dashboard named `<stack-name>-lambda`, with invocations, errors, duration percentiles, concurrency and recent log events.

## 8. Under the hood: many requests at once

These are the details that make the service correct when many requests arrive at once.

**All or nothing.** Registration writes three items, the user, their email login and a reservation of their name, in **one DynamoDB transaction**: either all three are saved, or none is. That's what keeps two people from registering the same name at the same moment. (A reservation's key is `NAME#<family-byte-length>:<family><userName>`. The length prefix keeps different name pairs from ever producing the same key.)

**Updating a user** needs the user's token. It keeps `createdAt` and the email unchanged, and checks a `revision` number: if someone else changed the record in the meantime, the update fails with `409 Conflict`, and the client should read the record again before retrying. A taken name also gives `409`.

**Input rules.** Names and emails are compared exactly, including upper and lower case. Name and family fields must be 1–100 bytes and not only spaces. The email check only looks at the basic shape of an address: it can't tell whether the mailbox exists.

**Reading fresh data.** Users, logins and refresh tokens are read with **strongly consistent** reads, which always see the latest write. DynamoDB's secondary indexes (GSIs) are only **eventually consistent**, a moment behind, and can't enforce uniqueness: that's why names use reservations instead. [DynamoDB transactions](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/transaction-apis.html), [read consistency](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.ReadConsistency.html)

**Refresh tokens.** A refresh token looks like `<user-UUID>.<random-UUID>`. The first half only says which user it's for; the random second half is the secret. Each user has one active refresh token:
- a new login replaces it, so if two logins happen at once, the one saved last wins;
- refreshing checks the whole old token before replacing it, so of two simultaneous refreshes only one succeeds;
- logging out deletes the token only if it's still the current one, so logging out with an old token can't log out a newer session.

**Password hashing** with Argon2 is deliberately slow, so it runs on a separate thread (`spawn_blocking`), at most two at a time per Lambda instance. For an unknown email, login still hashes a dummy password, so the response time gives fewer hints about which emails exist.

**Small details.** A token whose expiry time `exp` is the current second already counts as expired. At start-up, the code waits until each table is `ACTIVE`, and returns an error if that takes too long.

## 9. Moving existing data

New, empty tables need nothing. To use these handlers with **existing** data, first stop all writes (a maintenance window), then:
- create a name reservation for every existing user, after fixing any duplicate (family, name) pairs;
- make sure every user ID is a 36-character UUID, like the ones registration creates;
- delete the old-style refresh tokens, so users log in again.

A missing `revision` is fine: it's treated as the first version. Never run old and new handlers at the same time: the old ones would skip the reservation and token rules.

## 10. What the test results mean

- Without DynamoDB Local running, the integration tests are **skipped**, and say so; the unit tests still run. Run `REQUIRE_DYNAMODB=1 cargo test` to make a missing database a failure instead.
- With the database running, any problem creating the tables fails the test, never skips it. Every test deletes its tables afterwards, even when it fails or panics.
- The concurrency tests check the cases above: a losing registration leaves no half-created user, a failed rename keeps the old password, logging out with an old token keeps the newer one, and of two simultaneous refreshes only one wins.
