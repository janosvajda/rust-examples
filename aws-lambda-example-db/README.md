# aws-lambda-example-db

`aws-lambda-example-db` is a fully Rust, serverless user-management API designed
for AWS Lambda. It demonstrates end-to-end packaging with `cargo-lambda`,
persists identities in DynamoDB, and exposes login/token flows over an HTTP
interface implemented with `lambda_http`. The stack provisions application,
credentials, and refresh-token tables plus an API Gateway front door so you can
deploy a realistic auth service with minimal setup.

```
aws-lambda-example-db/
├─ src/            # Lambda entrypoint
├─ Cargo.toml      # Rust crate definition
├─ template.yaml   # SAM/CloudFormation template (works with cargo-lambda)
└─ README.md
```

## DynamoDB schema

Each deployment creates a table named `Users_<environment>`, for example
`Users_Prod` for production or `Users_Local` when running locally. The table
contains these attributes:

| Attribute  | Type   | Notes                                                    |
|------------|--------|----------------------------------------------------------|
| `userId`   | string | Partition key (auto-generated if absent)                |
| `userName` | string | Required display name; unique per family via GSI        |
| `email`    | string | Required email address; indexed via `EmailIndex`        |
| `familyId` | string | Required grouping id; indexed via `FamilyIdIndex`       |
| `createdAt`| string | RFC3339 timestamp                                       |
| `updatedAt`| string | RFC3339 timestamp                                       |

Credentials are stored separately in `UserCredentials_<env>` with `email` as
the partition key and attributes `userId`, `familyId`, and `passwordHash`.
Opaque refresh tokens live in `UserRefreshTokens_<env>` with attributes
`refreshToken` (partition key), `userId`, `familyId`, and `expiresAt`.

You can extend the schema by updating `UserRecord` in `src/user.rs` and the
`UserTable` resource inside `template.yaml`.

## Lambda API

| Method            | Purpose                                | Notes                                                                                                      |
|-------------------|----------------------------------------|------------------------------------------------------------------------------------------------------------|
| `POST /users`     | Create/upsert a user                   | Body: `{"userName": "...", "email": "...", "password": "...", "familyId": "...", "userId": "...?"}`. |
| `GET /users`      | Fetch a user by `userId`               | Requires `?userId=...` query parameter.                                                                    |
| `POST /login`     | Authenticate and mint JWT tokens       | Body: `{"email": "...", "password": "..."}`. Returns access + refresh tokens and metadata.             |
| `POST /token/refresh` | Exchange refresh token for new tokens | Body: `{"refreshToken": "..."}`. Rotates refresh token and returns a new access token pair.               |
| `POST /token/revoke`  | Revoke a refresh token              | Body: `{"refreshToken": "..."}`. Deletes the token; subsequent refresh attempts fail with 401.          |

`familyId` + `userName` pairs must be unique. Attempting to create a second
user with the same combination returns HTTP `409 Conflict`. Email addresses are
unique across the system. Passwords are hashed with Argon2 and stored in the
credentials table. Access tokens are signed using HS256 and default to a 15
minute lifetime; refresh tokens default to 7 days. Deploy the Lambda behind an
API Gateway private integration or IAM-authorised invocation so that only
services within the same AWS account/VPC can invoke the endpoints.

Responses are JSON encoded and include full user records. Passwords are stored
in plain text for simplicity—do **not** copy this behaviour for production use.

## Building the Lambda binary

```bash
cargo lambda build --release
```

Artifacts end up in `target/lambda/aws-lambda-example-db/`.

## Deploying with cargo-lambda

`cargo lambda deploy` uses the standard AWS credential chain. Explicit CLI
arguments (such as `--profile`) win first, followed by the `AWS_PROFILE`
environment variable, then direct access keys (`AWS_ACCESS_KEY_ID`, etc.), and
finally your default AWS CLI profile or instance role. Run
`aws sts get-caller-identity [--profile your-profile]` before deploying to verify
which AWS account the command will target.

Before deploying, create the JWT signing secret in AWS Systems Manager Parameter
Store. By default the stack looks for a `SecureString` at
`aws-lambda-example-db/<environment>/JWT_SECRET` (overrideable via
`JwtSecretParameterPrefix`). Seed each environment’s secret ahead of time, for
example:

```bash
aws ssm put-parameter \
  --name aws-lambda-example-db/Prod/JWT_SECRET \
  --type SecureString \
  --value 'replace-with-strong-secret' \
  --overwrite
```

Repeat the command for staging, dev, and any other environments so the Lambda
can retrieve the secret during startup.

```bash
cargo lambda deploy \
    --profile default \
    --template template.yaml \
    --config-env prod \
    --enable-function-url # optional: publish Function URL
```

Use the same profile you validated with `aws sts get-caller-identity` so the
deployment targets the expected AWS account and region.

The template provisions the DynamoDB table, IAM permissions, and the Lambda
function. Output values include the API Gateway endpoint for the `/users`
resource.

### Environment manifests

The project ships with a `samconfig.toml` that captures per-environment
parameters:

| Config env | Stack name                      | Parameter overrides          |
|------------|---------------------------------|------------------------------|
| `prod`     | `aws-lambda-example-db-prod`    | `EnvironmentName=Prod`       |
| `staging`  | `aws-lambda-example-db-staging` | `EnvironmentName=Staging`    |
| `local`    | `aws-lambda-example-db-local`   | `EnvironmentName=Local`      |

Pick the appropriate environment by passing `--config-env`. Adjust or extend the
file to match your AWS profiles, regions, or additional parameters (memory,
timeouts, alarms, etc.).

At runtime the handler respects the `ENVIRONMENT_NAME` variable. If it is not
present, the code falls back to `Local` when running under tooling like
`cargo lambda watch`, otherwise `Prod`. Supplying a custom name via the template
or environment (for example `Staging` or `Dev`) automatically produces the
corresponding DynamoDB table `Users_<name>`.

### Environment overview

| Environment | How to run/deploy                                                  | DynamoDB table         | Notes                           |
|-------------|--------------------------------------------------------------------|------------------------|---------------------------------|
| `Prod`      | `cargo lambda deploy --config-env prod`                            | `Users_Prod`           | Requires AWS credentials        |
| `Staging`   | `cargo lambda deploy --config-env staging`                         | `Users_Staging`        | Same code, isolated resources   |
| `Local`     | `cargo lambda watch --env-file env/local.env` *(or set var inline)* | `Users_Local`          | Works with local/remote tables  |
| custom name | `ENVIRONMENT_NAME=Dev cargo lambda deploy --template template.yaml` | `Users_Dev` (derived)  | Useful for ephemeral test envs  |

### DynamoDB Local

Spin up DynamoDB Local with Docker:

```bash
docker run --rm -p 8000:8000 amazon/dynamodb-local
```

## Local testing

Run the Lambda in a local emulator backed by the DynamoDB Local instance above:

```bash
cargo lambda watch --env-file env/local.env
```

The env file seeds every required variable (environment name, table overrides,
JWT secret pointers, etc.), so prefer that workflow to avoid drift. For deployed
environments the JWT secret should live in AWS Systems Manager Parameter Store
(`aws-lambda-example-db/<env>/JWT_SECRET` by default). 

Set `JWT_SECRET_PARAMETER` to that name; the Lambda fetches it with
`ssm:GetParameter`. If the lookup fails (e.g., running locally without access)
the code falls back to the `JWT_SECRET` env var so you can still test offline.


Table creation on startup only happens when `BOOTSTRAP_DYNAMODB_TABLES` is
truthy; the local env file enables it, while deployed stacks should leave the
variable unset so CloudFormation owns the DynamoDB lifecycle.

1. Start DynamoDB Local (see the section above) so all three tables exist:
   `Users_Local`, `UserCredentials_Local`, and `UserRefreshTokens_Local`.
2. Run `cargo lambda watch --env-file env/local.env` in one terminal; the
   command streams Lambda logs locally and reloads on code changes.
3. In a second terminal, exercise the endpoints with `curl` or a REST client:
   ```bash
   curl -X POST http://127.0.0.1:9000/users \
     -H 'content-type: application/json' \
     -d '{"userName":"alice","email":"alice@example.com","password":"secret","familyId":"fam-1"}'

   curl -X POST http://127.0.0.1:9000/login \
     -H 'content-type: application/json' \
     -d '{"email":"alice@example.com","password":"secret"}'

   curl -X POST http://127.0.0.1:9000/token/refresh \
     -H 'content-type: application/json' \
     -d '{"refreshToken":"<refresh-token-from-login>"}'

   curl -X POST http://127.0.0.1:9000/token/revoke \
     -H 'content-type: application/json' \
     -d '{"refreshToken":"<refresh-token-to-revoke>"}'
   ```
   Subsequent calls (e.g., `/token/refresh`, `/token/revoke`) can be tested
   the same way using the returned tokens. The Lambda logs include the resolved
   environment and table names each time it reloads.

The provided `env/local.env` sets the required environment variables (including
`DYNAMODB_ENDPOINT`, `AWS_ALLOW_HTTP`, `AWS_SDK_LOAD_CONFIG`,
`CREDENTIALS_TABLE_NAME`, `REFRESH_TOKEN_TABLE_NAME`, `JWT_SECRET_PARAMETER`,
`JWT_SECRET`, and `BOOTSTRAP_DYNAMODB_TABLES`) so the
SDK can talk to DynamoDB Local over HTTP without TLS. Integration tests rely on
the same variables and will skip automatically if DynamoDB is unreachable.

## Tests

Run unit and integration tests (integration test skips unless `DYNAMODB_ENDPOINT`
is set):

```bash
cargo test
```

The integration suite provisions all DynamoDB tables in-memory and covers:

- user creation, update, duplicate rejection, multi-user families, and GSI reads (`tests/user_flow.rs`)
- login success/failure plus refresh token issuance (`tests/login_flow.rs`)
- JWT claim structure and signature verification (`tests/auth_flow.rs`)
- refresh token rotation and revocation (`tests/refresh_flow.rs`)
