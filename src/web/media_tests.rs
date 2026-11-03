//! Offline proxy response regressions using test-only DNS overrides, never an HTTP proxy.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::routing::get;
use http_body_util::BodyExt;
use tower::ServiceExt;

use super::{AppState, image_proxy, router};
use crate::aggregator::Searcher;
use crate::media_fetch::public_media_client;
use crate::network::Network;
use crate::settings::Settings;

#[tokio::test]
async fn image_and_favicon_proxies_sandbox_svg_without_changing_image_bytes() {
    for (upstream_type, content_type, payload) in [
        (
            "image/svg+xml",
            "image/svg+xml",
            r#"<svg xmlns="http://www.w3.org/2000/svg" onload="alert(document.cookie)"><script>alert(1)</script><rect width="10" height="10"/></svg>"#,
        ),
        ("image/png", "image/png", "raster image bytes"),
        ("Image/SVG+XML; charset=utf-8", "image/svg+xml", "<svg/>"),
    ] {
        let upstream =
            Router::new().fallback(get(move |headers: axum::http::HeaderMap| async move {
                assert_eq!(headers[header::ACCEPT], "image/*");
                ([(header::CONTENT_TYPE, upstream_type)], payload)
            }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, upstream).await.unwrap() });
        let mut settings = Settings::defaults();
        settings.engines.clear();
        settings.server.image_proxy = true;
        settings.server.secret_key = "test".into();
        let client = public_media_client(&settings.outgoing, &[("example.com".into(), address)])
            .unwrap()
            .0;
        let mut searcher = Searcher::new(settings.clone()).unwrap();
        searcher.network = Arc::new(
            Network::from_settings(&settings)
                .unwrap()
                .with_test_media_client(client),
        );
        let app = router(AppState::new(searcher));
        for (path, suffix, cache_seconds) in [
            ("image_proxy", "/image", 86_400),
            ("favicon_proxy", "", 604_800),
        ] {
            let url = format!("http://example.com:{}{suffix}", address.port());
            let uri = format!(
                "/{path}?url={}&h={}",
                crate::engine::util::encode(&url),
                image_proxy::signature("test", &url)
            );
            let response = app
                .clone()
                .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let headers = response.headers();
            assert_eq!(headers[header::CONTENT_TYPE], content_type);
            assert_eq!(
                headers[header::CACHE_CONTROL],
                format!("max-age={cache_seconds}")
            );
            assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
            assert_eq!(headers[header::REFERRER_POLICY], "no-referrer");
            assert_eq!(
                headers[header::CONTENT_SECURITY_POLICY],
                "sandbox; default-src 'none'; style-src 'unsafe-inline'; img-src data:; base-uri 'none'; form-action 'none'"
            );
            assert_eq!(
                response.into_body().collect().await.unwrap().to_bytes(),
                payload
            );
        }
        server.abort();
    }
}
