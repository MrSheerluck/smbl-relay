use reqwest::blocking::{Body, Client, Response};
use serde_json::{Value, json};
use std::{
    io::Cursor,
    sync::{Arc, Barrier},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn endpoint() -> String {
    let base =
        std::env::var("WAITLIST_TEST_URL").unwrap_or_else(|_| "http://localhost:8787".into());
    let url = url::Url::parse(&base).unwrap();
    assert!(
        matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")),
        "Tests require a local database"
    );
    url.join("/api/waitlist").unwrap().to_string()
}

fn client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap()
}
fn address(label: &str) -> String {
    format!(
        "relay-test-{}-{label}@example.com",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}
fn post(email: Value) -> Response {
    client()
        .post(endpoint())
        .header("Accept", "application/json")
        .json(&json!({"email": email}))
        .send()
        .unwrap()
}
fn code(response: Response, status: u16, expected: &str) {
    assert_eq!(response.status().as_u16(), status);
    assert_eq!(response.json::<Value>().unwrap()["code"], expected);
}

#[test]
#[ignore = "requires Wrangler and a migrated local D1 database"]
fn normalized_duplicates() {
    let email = address("normalize");
    code(
        post(json!(format!("  {}  ", email.to_uppercase()))),
        201,
        "joined",
    );
    for variant in [email.clone(), email.to_uppercase(), format!(" {email} ")] {
        code(post(json!(variant)), 200, "already_joined");
    }
}

#[test]
#[ignore = "requires Wrangler and a migrated local D1 database"]
fn simultaneous_submissions() {
    let email = address("race");
    let barrier = Arc::new(Barrier::new(12));
    let threads: Vec<_> = (0..12)
        .map(|i| {
            let email = if i % 2 == 0 {
                email.clone()
            } else {
                email.to_uppercase()
            };
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                let response = post(json!(email));
                let status = response.status().as_u16();
                code(
                    response,
                    status,
                    if status == 201 {
                        "joined"
                    } else {
                        "already_joined"
                    },
                );
                status
            })
        })
        .collect();
    let statuses: Vec<_> = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();
    assert_eq!(statuses.iter().filter(|&&s| s == 201).count(), 1);
    assert_eq!(statuses.iter().filter(|&&s| s == 200).count(), 11);
}

#[test]
#[ignore = "requires Wrangler and a migrated local D1 database"]
fn invalid_input() {
    for email in [
        json!(""),
        json!("not-an-email"),
        json!("a@localhost"),
        json!("a..b@example.com"),
        json!(".a@example.com"),
        json!("a.@example.com"),
        json!("a@-example.com"),
        json!("a@ex ample.com"),
        json!(format!("{}@example.com", "a".repeat(65))),
        json!(42),
        Value::Null,
    ] {
        code(post(email), 400, "invalid_email");
    }
    code(
        client()
            .post(endpoint())
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .body("{")
            .send()
            .unwrap(),
        400,
        "invalid_request",
    );
}

#[test]
#[ignore = "requires Wrangler and a migrated local D1 database"]
fn oversized_body() {
    code(post(json!("a".repeat(3000))), 413, "body_too_large");
}

#[test]
#[ignore = "requires Wrangler and a migrated local D1 database"]
fn chunked_body() {
    // A reader with unknown size sends chunked HTTP without Content-Length.
    let body = Body::new(Cursor::new(
        json!({"email": "a".repeat(3000)}).to_string().into_bytes(),
    ));
    code(
        client()
            .post(endpoint())
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .unwrap(),
        413,
        "body_too_large",
    );
}

#[test]
#[ignore = "requires Wrangler and a migrated local D1 database"]
fn request_guards() {
    code(
        client()
            .post(endpoint())
            .header("Accept", "application/json")
            .header("Origin", "https://elsewhere.example")
            .json(&json!({"email": address("origin")}))
            .send()
            .unwrap(),
        403,
        "invalid_origin",
    );
    code(
        client()
            .post(endpoint())
            .header("Accept", "application/json")
            .header("Sec-Fetch-Site", "cross-site")
            .json(&json!({"email": address("fetch-site")}))
            .send()
            .unwrap(),
        403,
        "invalid_origin",
    );
    code(
        client()
            .post(endpoint())
            .header("Accept", "application/json")
            .header("Content-Type", "text/plain")
            .body("email=a@example.com")
            .send()
            .unwrap(),
        415,
        "unsupported_format",
    );
    let response = client()
        .get(endpoint())
        .header("Accept", "application/json")
        .send()
        .unwrap();
    assert_eq!(response.headers()["Allow"], "POST");
    code(response, 405, "method_not_allowed");
}

#[test]
#[ignore = "requires Wrangler and a migrated local D1 database"]
fn plain_html_form() {
    let email = address("no-js");
    let origin = url::Url::parse(&endpoint())
        .unwrap()
        .origin()
        .ascii_serialization();
    let body: String = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("email", &email)
        .finish();
    for (status, message) in [
        (201, "You’re on the waitlist"),
        (200, "already on the waitlist"),
    ] {
        let response = client()
            .post(endpoint())
            .header("Accept", "text/html")
            .header("Origin", &origin)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.clone())
            .send()
            .unwrap();
        assert_eq!(response.status().as_u16(), status);
        assert!(
            response.headers()["Content-Type"]
                .to_str()
                .unwrap()
                .starts_with("text/html")
        );
        assert!(response.text().unwrap().contains(message));
    }
}
