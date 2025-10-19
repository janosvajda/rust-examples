use std::{borrow::Cow, env, fmt};

const ENVIRONMENT_NAME_ENV: &str = "ENVIRONMENT_NAME";
const DEFAULT_REMOTE_ENVIRONMENT: &str = "Prod";
const DEFAULT_LOCAL_ENVIRONMENT: &str = "Local";

#[derive(Debug, Clone, Copy)]
pub enum ResolutionSource {
    ExplicitVar,
    LocalTooling,
    AwsRuntime,
    DefaultLocal,
}

impl fmt::Display for ResolutionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolutionSource::ExplicitVar => write!(f, "explicit ENVIRONMENT_NAME"),
            ResolutionSource::LocalTooling => write!(f, "local tooling auto-detect"),
            ResolutionSource::AwsRuntime => write!(f, "AWS runtime auto-detect"),
            ResolutionSource::DefaultLocal => write!(f, "fallback to Local"),
        }
    }
}

/// Encapsulates the deployment environment (Prod, Staging, Local, ...).
///
/// The detection order is:
///  1. Explicit `ENVIRONMENT_NAME` (set via `samconfig.toml`, CI, or CLI)
///  2. Local tooling hints (`cargo lambda watch`, SAM local, LocalStack)
///  3. AWS Lambda runtime heuristics
///  4. Default to `Local`
pub struct DeploymentEnv {
    name: Cow<'static, str>,
    source: ResolutionSource,
}

impl DeploymentEnv {
    pub fn detect() -> Self {
        if let Some(explicit) = Self::explicit_override() {
            return explicit;
        }
        if Self::running_locally() {
            return Self::local(ResolutionSource::LocalTooling);
        }
        if Self::running_on_aws() {
            return Self::remote(ResolutionSource::AwsRuntime);
        }
        Self::local(ResolutionSource::DefaultLocal)
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn table_name(&self) -> String {
        format!("Users_{}", self.name())
    }

    pub fn source(&self) -> ResolutionSource {
        self.source
    }

    fn explicit_override() -> Option<Self> {
        env::var(ENVIRONMENT_NAME_ENV).ok().and_then(|raw| {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(Self {
                    name: Cow::Owned(trimmed.to_owned()),
                    source: ResolutionSource::ExplicitVar,
                })
            }
        })
    }

    fn running_locally() -> bool {
        env::var_os("AWS_SAM_LOCAL").is_some()
            || env::var_os("CARGO_LAMBDA_HTTP_PORT").is_some()
            || env::var_os("LOCALSTACK_HOSTNAME").is_some()
    }

    fn running_on_aws() -> bool {
        env::var_os("AWS_EXECUTION_ENV").is_some()
            || env::var_os("AWS_REGION").is_some()
            || env::var_os("AWS_LAMBDA_FUNCTION_NAME").is_some()
            || env::var_os("LAMBDA_TASK_ROOT").is_some()
    }

    fn remote(source: ResolutionSource) -> Self {
        Self {
            name: Cow::Borrowed(DEFAULT_REMOTE_ENVIRONMENT),
            source,
        }
    }

    fn local(source: ResolutionSource) -> Self {
        Self {
            name: Cow::Borrowed(DEFAULT_LOCAL_ENVIRONMENT),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn clear_env_vars() {
        for key in [
            "ENVIRONMENT_NAME",
            "AWS_SAM_LOCAL",
            "CARGO_LAMBDA_HTTP_PORT",
            "LOCALSTACK_HOSTNAME",
            "AWS_EXECUTION_ENV",
            "AWS_REGION",
            "AWS_LAMBDA_FUNCTION_NAME",
            "LAMBDA_TASK_ROOT",
        ] {
            std::env::remove_var(key);
        }
    }

    #[test]
    #[serial]
    fn explicit_override_wins() {
        clear_env_vars();
        std::env::remove_var("ENVIRONMENT_NAME");
        std::env::set_var("ENVIRONMENT_NAME", "Staging");
        let env = DeploymentEnv::detect();
        assert_eq!(env.name(), "Staging");
        assert_eq!(env.table_name(), "Users_Staging");
        matches!(env.source(), ResolutionSource::ExplicitVar);
        std::env::remove_var("ENVIRONMENT_NAME");
    }

    #[test]
    #[serial]
    fn local_tooling_fallback() {
        clear_env_vars();
        std::env::remove_var("ENVIRONMENT_NAME");
        std::env::set_var("AWS_SAM_LOCAL", "1");
        let env = DeploymentEnv::detect();
        assert_eq!(env.name(), "Local");
        assert_eq!(env.table_name(), "Users_Local");
        matches!(env.source(), ResolutionSource::LocalTooling);
        std::env::remove_var("AWS_SAM_LOCAL");
    }

    #[test]
    #[serial]
    fn aws_runtime_fallback() {
        clear_env_vars();
        std::env::set_var("AWS_EXECUTION_ENV", "AWS_Lambda_rust");
        let env = DeploymentEnv::detect();
        assert_eq!(env.name(), DEFAULT_REMOTE_ENVIRONMENT);
        assert_eq!(env.table_name(), "Users_Prod");
        matches!(env.source(), ResolutionSource::AwsRuntime);
        std::env::remove_var("AWS_EXECUTION_ENV");
    }
}
