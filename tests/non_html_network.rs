//! Regression coverage for rejecting HTML before response-body consumption.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use axum::Router;
use axum::body::{Body, Bytes};
use axum::http::{Response, StatusCode, header};
use axum::response::Redirect;
use axum::routing::get;
use saferustxng::cache::{Cache, CachedResponse};
use saferustxng::deep_research::LLMClient;
use saferustxng::engine::{EngineRequest, FailureKind};
use saferustxng::network::{Network, NetworkError, validate_machine_content_type};
use saferustxng::settings::Settings;

async fn response_server(
    content_type: Option<&'static str>,
    status: StatusCode,
    body: Option<&'static str>,
) -> (String, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    let requests = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&requests);
    let app = Router::new()
        .route("/redirect", get(|| async { Redirect::temporary("/") }))
        .fallback(move || {
            counter.fetch_add(1, Ordering::SeqCst);
            async move {
                let mut response = Response::builder().status(status);
                if let Some(content_type) = content_type {
                    response = response.header(header::CONTENT_TYPE, content_type);
                }
                let body = match body {
                    Some(body) => Body::from(body),
                    None => Body::from_stream(futures::stream::pending::<
                        Result<Bytes, std::io::Error>,
                    >()),
                };
                response.body(body).unwrap()
            }
        });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (url, requests, server)
}

#[test]
fn machine_content_types_are_explicit_and_html_is_not_retryable() {
    for value in [
        "application/json",
        "Application/JSON; charset=UTF-8",
        "application/problem+json",
        "application/rss+xml",
        "application/atom+xml",
        "application/rdf+xml",
        "text/xml",
        "text/plain",
        "text/event-stream",
        "application/javascript",
        "application/x-ndjson",
        "image/png",
        "image/svg+xml",
    ] {
        assert!(
            validate_machine_content_type(Some(value)).is_ok(),
            "{value}"
        );
    }
    for value in [
        None,
        Some(""),
        Some("text/html"),
        Some("TEXT/HTML; charset=utf-8"),
        Some("application/xhtml+xml"),
        Some("application/octet-stream"),
    ] {
        let error = validate_machine_content_type(value).unwrap_err();
        assert!(matches!(error, NetworkError::UnsupportedContentType(_)));
        assert_eq!(error.failure_kind(), FailureKind::Parse);
    }
}

#[tokio::test]
async fn html_is_rejected_before_waiting_for_its_body_or_retrying() {
    for content_type in [
        Some("text/html"),
        Some("TEXT/HTML; charset=utf-8"),
        Some("application/xhtml+xml"),
        None,
    ] {
        for status in [
            StatusCode::OK,
            StatusCode::FORBIDDEN,
            StatusCode::TOO_MANY_REQUESTS,
        ] {
            // A body that never arrives proves rejection happens at the headers.
            let (url, requests, server) = response_server(content_type, status, None).await;
            let mut settings = Settings::defaults();
            settings.outgoing.retries = 0;
            let network = Network::from_settings(&settings).unwrap();
            let error = network
                .fetch(EngineRequest::get(url), Duration::from_secs(1), None, None)
                .await
                .unwrap_err();
            if status == StatusCode::OK {
                assert!(matches!(error, NetworkError::UnsupportedContentType(_)));
            } else {
                assert!(matches!(&error, NetworkError::Status { body, response, .. }
                    if body.is_empty() && response.body.is_empty()));
                let (kind, suspension) = if status == StatusCode::FORBIDDEN {
                    (FailureKind::AccessDenied, 86_400)
                } else {
                    (FailureKind::TooManyRequests, 300)
                };
                assert_eq!(error.failure_kind(), kind);
                let states = saferustxng::engine::state::EngineStates::default();
                assert_eq!(
                    states.record_failure("test", kind, &settings.search),
                    suspension
                );
            }
            assert_eq!(requests.load(Ordering::SeqCst), 1);
            server.abort();
        }
    }
}

#[tokio::test]
async fn http_failures_retry_with_or_without_machine_content_types() {
    for content_type in [Some("text/html"), None, Some("application/json")] {
        for status in [
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::SERVICE_UNAVAILABLE,
        ] {
            // Rejected bodies never arrive; supported JSON errors can be consumed.
            let body = (content_type == Some("application/json")).then_some("{}");
            let (url, requests, server) = response_server(content_type, status, body).await;
            let mut settings = Settings::defaults();
            settings.outgoing.retries = 1;
            let network = Network::from_settings(&settings).unwrap();
            let error = network
                .fetch(EngineRequest::get(url), Duration::from_secs(3), None, None)
                .await
                .unwrap_err();
            assert!(matches!(error, NetworkError::Status { status: code, .. }
                if code == status.as_u16()));
            assert_eq!(requests.load(Ordering::SeqCst), 2);
            server.abort();
        }
    }
}

#[tokio::test]
async fn rate_limit_retry_can_recover_to_a_machine_response() {
    let requests = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&requests);
    let app = Router::new().fallback(move || {
        let attempt = counter.fetch_add(1, Ordering::SeqCst);
        async move {
            if attempt == 0 {
                Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(Body::from_stream(futures::stream::pending::<
                        Result<Bytes, std::io::Error>,
                    >()))
                    .unwrap()
            } else {
                Response::builder()
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{}"))
                    .unwrap()
            }
        }
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let mut settings = Settings::defaults();
    settings.outgoing.retries = 1;
    let network = Network::from_settings(&settings).unwrap();
    let response = network
        .fetch(EngineRequest::get(url), Duration::from_secs(3), None, None)
        .await
        .unwrap();
    assert_eq!(response.body, b"{}");
    assert_eq!(requests.load(Ordering::SeqCst), 2);
    server.abort();
}

#[tokio::test]
async fn api_feeds_text_and_images_remain_available() {
    for (content_type, body) in [
        ("application/json", "{\"results\":[]}"),
        ("application/atom+xml", "<feed/>"),
        ("application/rss+xml", "<rss/>"),
        ("text/plain", "ExitAddress 203.0.113.1"),
        ("text/event-stream", "data: {\"results\":[]}\n\n"),
        ("application/javascript", "callback({});"),
        ("image/png", "image bytes"),
    ] {
        let (url, _, server) =
            response_server(Some(content_type), StatusCode::OK, Some(body)).await;
        let network = Network::from_settings(&Settings::defaults()).unwrap();
        let response = network
            .fetch(EngineRequest::get(url), Duration::from_secs(1), None, None)
            .await
            .unwrap();
        assert_eq!(response.body, body.as_bytes());
        server.abort();
    }
}

#[tokio::test]
async fn custom_html_accept_headers_are_rejected_without_network_access() {
    let (url, requests, server) = response_server(Some("text/html"), StatusCode::OK, None).await;
    let network = Network::from_settings(&Settings::defaults()).unwrap();
    for accept in [
        "text/html",
        "application/json, TEXT/HTML; q=0.8",
        "application/xhtml+xml",
    ] {
        let error = network
            .fetch(
                EngineRequest::get(&url).header("Accept", accept),
                Duration::from_secs(1),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(matches!(error, NetworkError::Build(_)));
    }
    assert_eq!(requests.load(Ordering::SeqCst), 0);
    server.abort();
}

#[tokio::test]
async fn redirects_cannot_bypass_html_rejection() {
    let (url, _, server) = response_server(Some("text/html"), StatusCode::OK, None).await;
    let network = Network::from_settings(&Settings::defaults()).unwrap();
    let error = network
        .fetch(
            EngineRequest::get(format!("{url}redirect")),
            Duration::from_secs(1),
            None,
            None,
        )
        .await
        .unwrap_err();
    assert!(matches!(error, NetworkError::UnsupportedContentType(_)));
    server.abort();
}

#[tokio::test]
async fn cached_html_is_not_returned_to_engines() {
    let cache = Cache::memory(Duration::from_secs(60), 4096);
    let url = "http://127.0.0.1:1/never-requested";
    cache
        .set(
            &Cache::key("GET", url, ""),
            CachedResponse {
                status: 200,
                final_url: url.into(),
                headers: [("content-type".into(), "text/html".into())].into(),
                body: Arc::new(b"<html>stale cache entry</html>".to_vec()),
            },
        )
        .await;
    let network = Network::from_settings(&Settings::defaults()).unwrap();
    let error = network
        .fetch(
            EngineRequest::get(url),
            Duration::from_secs(1),
            Some(&cache),
            None,
        )
        .await
        .unwrap_err();
    assert!(matches!(error, NetworkError::UnsupportedContentType(_)));
}

#[tokio::test]
async fn llm_client_rejects_html_before_reading_a_body() {
    let (url, _, server) = response_server(Some("text/html"), StatusCode::OK, None).await;
    let client = LLMClient::new(&url, "test", "test", Duration::from_secs(1)).unwrap();
    let error = client.complete("system", "question").await.unwrap_err();
    assert!(error.contains("machine response required"), "{error}");
    server.abort();
}
