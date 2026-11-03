//! Regression coverage for removed sources and legacy autocomplete preferences.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use saferustxng::aggregator::Searcher;
use saferustxng::autocomplete::{BACKENDS, suggest};
use saferustxng::network::Network;
use saferustxng::settings::Settings;
use saferustxng::web::{AppState, router};
use tower::ServiceExt;

#[tokio::test]
async fn removed_autocomplete_backends_never_open_network_connections() {
    // Every outbound connection goes to this local proxy, including HTTPS CONNECT.
    // Drop accepted sockets so even positive controls never reach an external host.
    let connections = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&connections);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let proxy = tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            counter.fetch_add(1, Ordering::SeqCst);
            drop(socket);
        }
    });
    let mut settings = Settings::defaults();
    settings
        .outgoing
        .proxies
        .insert("https".into(), format!("http://{address}"));
    settings.outgoing.retries = 0;
    settings.search.timeout_limit = 1.0;
    let network = Network::from_settings(&settings).unwrap();

    // Prove the trap detects each supported backend instead of silently bypassing it.
    for backend in BACKENDS {
        let before = connections.load(Ordering::SeqCst);
        assert!(
            suggest(&network, &settings, backend, "rust", "en")
                .await
                .is_empty()
        );
        assert!(connections.load(Ordering::SeqCst) > before, "{backend}");
    }
    connections.store(0, Ordering::SeqCst);
    for backend in ["google", "unknown", "none", ""] {
        assert!(
            suggest(&network, &settings, backend, "rust", "en")
                .await
                .is_empty()
        );
    }

    // Exercise query parameters, old cookies, and an old instance setting.
    for configured_backend in ["duckduckgo", "google", "unknown"] {
        settings.search.autocomplete = configured_backend.into();
        let app = router(AppState::new(Searcher::new(settings.clone()).unwrap()));
        let mut cases = vec![
            ("/autocompleter?q=rust&backend=google", None),
            ("/autocompleter?q=rust&backend=unknown", None),
            ("/autocompleter?q=rust&backend=google,unknown", None),
            ("/autocompleter?q=rust", Some("autocomplete=google")),
            ("/autocompleter?q=rust", Some("autocomplete=unknown")),
        ];
        if configured_backend != "duckduckgo" {
            cases.push(("/autocompleter?q=rust", None));
        }
        for (uri, cookie) in cases {
            let mut request = Request::builder().uri(uri);
            if let Some(cookie) = cookie {
                request = request.header(header::COOKIE, cookie);
            }
            let response = app
                .clone()
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let body = response.into_body().collect().await.unwrap().to_bytes();
            let suggestions: Vec<String> = serde_json::from_slice(&body).unwrap();
            assert!(suggestions.is_empty(), "{configured_backend}: {uri}");
        }
        assert_eq!(
            connections.load(Ordering::SeqCst),
            0,
            "{configured_backend}"
        );
    }
    proxy.abort();
}

#[tokio::test]
async fn config_and_preferences_expose_only_retained_defaults() {
    let settings = Settings::defaults();
    let expected_names: Vec<_> = settings
        .enabled_engines()
        .map(|config| config.name.clone())
        .collect();
    let app = router(AppState::new(Searcher::new(settings).unwrap()));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let config: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let names: Vec<_> = config["engines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|engine| engine["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(names, expected_names);
    assert_eq!(config["autocomplete"], "duckduckgo");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/preferences")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("value=\"duckduckgo\""));
    assert!(!html.contains("value=\"google\""));
}
