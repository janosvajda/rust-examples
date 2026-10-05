use lambda_http::{Body, Error, Request, RequestExt, Response};

/// Greets whoever is named in `?name=…`.
///
/// The name comes from the caller, so it's untrusted input. The response is
/// sent as plain text: a browser shows `<script>` as those characters, instead
/// of running it. (As `text/html`, `?name=<script>…</script>` would run that
/// script in the visitor's browser: a "reflected XSS" attack.)
pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let who = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("name"))
        .unwrap_or("world");
    // Keep untrusted input short: nobody's name is longer than this.
    let who: String = who.chars().take(100).collect();
    let message = format!("Hello {who}, this is an AWS Lambda HTTP request");

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/plain; charset=utf-8")
        .body(message.into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::{Request, RequestExt};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_generic_http_handler() {
        let request = Request::default();

        let response = function_handler(request).await.unwrap();
        assert_eq!(response.status(), 200);

        let body_bytes = response.body().to_vec();
        let body_string = String::from_utf8(body_bytes).unwrap();

        assert_eq!(
            body_string,
            "Hello world, this is an AWS Lambda HTTP request"
        );
    }

    #[tokio::test]
    async fn test_http_handler_with_query_string() {
        let mut query_string_parameters: HashMap<String, String> = HashMap::new();
        query_string_parameters.insert("name".into(), "aws-lambda-example-hello-world".into());

        let request = Request::default().with_query_string_parameters(query_string_parameters);

        let response = function_handler(request).await.unwrap();
        assert_eq!(response.status(), 200);

        let body_bytes = response.body().to_vec();
        let body_string = String::from_utf8(body_bytes).unwrap();

        assert_eq!(
            body_string,
            "Hello aws-lambda-example-hello-world, this is an AWS Lambda HTTP request"
        );
    }

    #[tokio::test]
    async fn html_in_the_name_is_never_served_as_html() {
        let attack = "<script>alert('hi')</script>";
        let request = Request::default().with_query_string_parameters(HashMap::from([(
            "name".to_string(),
            attack.to_string(),
        )]));
        let response = function_handler(request).await.unwrap();
        // Plain text: the browser displays these characters, it doesn't run them.
        assert_eq!(
            response.headers()["content-type"],
            "text/plain; charset=utf-8"
        );
    }

    #[tokio::test]
    async fn very_long_names_are_cut_short() {
        let request = Request::default().with_query_string_parameters(HashMap::from([(
            "name".to_string(),
            "x".repeat(10_000),
        )]));
        let response = function_handler(request).await.unwrap();
        assert!(response.body().len() < 200);
    }
}
