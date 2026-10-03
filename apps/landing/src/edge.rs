use futures_util::StreamExt;
use serde::Serialize;
use wasm_bindgen::JsValue;
use worker::{Context, Env, Method, Request, Response, Result, event};

const CSP: &str = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'";
const MAX_BODY: usize = 2048;

#[event(fetch)]
pub async fn main(mut request: Request, env: Env, _ctx: Context) -> Result<Response> {
    match request.path().as_str() {
        "/api/waitlist" => waitlist(&mut request, &env).await,
        "/" => {
            if !matches!(request.method(), Method::Get | Method::Head) {
                let mut response = Response::error("Method not allowed", 405)?;
                response.headers_mut().set("Allow", "GET, HEAD")?;
                return Ok(response);
            }
            let mut response = if request.method() == Method::Head {
                Response::empty()?
            } else {
                match crate::render().await {
                    Ok(html) => Response::from_html(html)?,
                    Err(_) => {
                        worker::console_error!("landing_render_failed");
                        return Response::error(
                            "Could not render the page. Please try again.",
                            500,
                        );
                    }
                }
            };
            response
                .headers_mut()
                .set("Content-Type", "text/html; charset=utf-8")?;
            security(&mut response)?;
            Ok(response)
        }
        _ => env.assets("ASSETS")?.fetch_request(request).await,
    }
}

fn security(response: &mut Response) -> Result<()> {
    response
        .headers_mut()
        .set("X-Content-Type-Options", "nosniff")?;
    response
        .headers_mut()
        .set("Referrer-Policy", "strict-origin-when-cross-origin")?;
    response.headers_mut().set("Content-Security-Policy", CSP)
}

#[derive(Serialize)]
struct Reply<'a> {
    code: &'a str,
    message: &'a str,
}

fn reply(request: &Request, status: u16, code: &str, message: &str) -> Result<Response> {
    let mut response = if request.headers().get("Accept")?.unwrap_or_default().contains("application/json") {
        Response::from_json(&Reply { code, message })?
    } else {
        // Messages are fixed application strings; user input never enters this HTML.
        Response::from_html(format!("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Relay waitlist</title><link rel=\"stylesheet\" href=\"/style.css\"></head><body><main class=\"waitlist-result\"><a class=\"brand\" href=\"/\"><img class=\"brand-mark\" src=\"/relay-mark.svg\" alt=\"\" width=\"36\" height=\"36\">relay</a><h1>Relay waitlist</h1><p>{message}</p><a class=\"button primary\" href=\"/#waitlist\">Back to Relay</a></main></body></html>"))?
    }.with_status(status);
    response.headers_mut().set("Cache-Control", "no-store")?;
    security(&mut response)?;
    Ok(response)
}

async fn waitlist(request: &mut Request, env: &Env) -> Result<Response> {
    if request.method() != Method::Post {
        let mut response = reply(
            request,
            405,
            "method_not_allowed",
            "Use the email form to join the waitlist.",
        )?;
        response.headers_mut().set("Allow", "POST")?;
        return Ok(response);
    }
    let origin = request.url()?.origin().ascii_serialization();
    if request
        .headers()
        .get("Origin")?
        .is_some_and(|value| value != origin)
        || request.headers().get("Sec-Fetch-Site")?.as_deref() == Some("cross-site")
    {
        return reply(
            request,
            403,
            "invalid_origin",
            "Please join the waitlist from the Relay website.",
        );
    }
    let content_type = request.headers().get("Content-Type")?.unwrap_or_default();
    let content_type = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if !matches!(
        content_type.as_str(),
        "application/json" | "application/x-www-form-urlencoded"
    ) {
        return reply(
            request,
            415,
            "unsupported_format",
            "Use the email form to join the waitlist.",
        );
    }
    let too_large = || {
        reply(
            request,
            413,
            "body_too_large",
            "The form is too large. Enter just your email address.",
        )
    };
    if request
        .headers()
        .get("Content-Length")?
        .and_then(|v| v.parse::<usize>().ok())
        .is_some_and(|size| size > MAX_BODY)
    {
        return too_large();
    }
    let mut body = Vec::new();
    let Ok(mut stream) = request.stream() else {
        return reply(
            request,
            400,
            "invalid_request",
            "Enter a valid email address.",
        );
    };
    while let Some(chunk) = stream.next().await {
        let Ok(chunk) = chunk else {
            return reply(
                request,
                400,
                "invalid_request",
                "Enter a valid email address.",
            );
        };
        if body.len() + chunk.len() > MAX_BODY {
            // Dropping ByteStream releases its reader and cancels the remaining body.
            drop(stream);
            return reply(
                request,
                413,
                "body_too_large",
                "The form is too large. Enter just your email address.",
            );
        }
        body.extend_from_slice(&chunk);
    }
    let email = if content_type == "application/json" {
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&body) else {
            return reply(
                request,
                400,
                "invalid_request",
                "Enter a valid email address.",
            );
        };
        value
            .get("email")
            .and_then(|v| v.as_str())
            .map(str::to_owned)
    } else {
        url::form_urlencoded::parse(&body)
            .find(|(key, _)| key == "email")
            .map(|(_, value)| value.into_owned())
    };
    let Some(email) = email.as_deref().and_then(crate::email::normalize) else {
        return reply(
            request,
            400,
            "invalid_email",
            "Enter a valid email address.",
        );
    };
    let insert = async {
        env.d1("DB")?
            .prepare("INSERT INTO waitlist (email) VALUES (?1) ON CONFLICT(email) DO NOTHING")
            .bind(&[JsValue::from_str(&email)])?
            .run()
            .await
    }
    .await;
    match insert {
        Ok(result) if result.success() => match result.meta()?.and_then(|meta| meta.changes) {
            Some(0) => reply(
                request,
                200,
                "already_joined",
                "You’re already on the waitlist. We’ll email you when Relay is ready.",
            ),
            Some(_) => reply(
                request,
                201,
                "joined",
                "You’re on the waitlist. We’ll email you when Relay is ready.",
            ),
            None => unavailable(request),
        },
        _ => unavailable(request),
    }
}

fn unavailable(request: &Request) -> Result<Response> {
    worker::console_error!("waitlist_insert_failed");
    reply(
        request,
        503,
        "temporarily_unavailable",
        "The waitlist is temporarily unavailable. Please try again in a moment.",
    )
}
