//! Search suggestions, ported from `searx/autocomplete.py`.

use std::collections::BTreeSet;

use crate::engine::util::{json_str, locale_language};
use crate::engine::{EngineRequest, EngineResponse};
use crate::network::Network;
use crate::settings::Settings;

/// Autocomplete backends, in the order they are tried.
pub static BACKENDS: &[&str] = &["duckduckgo", "wikipedia", "mwmbl"];

/// Fetch suggestions from a supported backend; unknown names never send queries.
pub async fn suggest(
    network: &Network,
    settings: &Settings,
    backend: &str,
    query: &str,
    locale: &str,
) -> Vec<String> {
    let url = match backend {
        "wikipedia" => {
            let language = locale_language(locale);
            let language = if language == "all" {
                "en".to_string()
            } else {
                language
            };
            format!(
                "https://{language}.wikipedia.org/w/api.php?action=opensearch&limit=10&format=json&search={}",
                crate::engine::util::encode(query)
            )
        }
        "mwmbl" => format!(
            "https://api.mwmbl.org/api/v1/search/?s={}",
            crate::engine::util::encode(query)
        ),
        "duckduckgo" => format!(
            "https://duckduckgo.com/ac/?type=list&q={}",
            crate::engine::util::encode(query)
        ),
        _ => return Vec::new(),
    };
    let request = EngineRequest::get(url).header("Accept", "application/json");
    let timeout = std::time::Duration::from_secs_f64(settings.search.timeout_limit.clamp(1.0, 3.0));
    match network.fetch(request, timeout, None, None).await {
        Ok(response) => parse_suggestions(backend, &response),
        Err(error) => {
            tracing::debug!(backend, %error, "autocomplete backend failed");
            Vec::new()
        }
    }
}

/// Extract the suggestions from a backend response.
pub fn parse_suggestions(backend: &str, response: &EngineResponse) -> Vec<String> {
    let Ok(json) = response.json() else {
        return Vec::new();
    };
    let mut suggestions = BTreeSet::new();
    match backend {
        "wikipedia" => {
            if let Some(items) = json_path_array(&json, 1) {
                for item in items {
                    if let Some(text) = item.as_str() {
                        suggestions.insert(text.to_string());
                    }
                }
            }
        }
        "mwmbl" => {
            if let Some(items) = json.as_array() {
                for item in items.iter().take(10) {
                    let title = json_str(item, "title.0.value");
                    if !title.is_empty() {
                        suggestions.insert(title);
                    }
                }
            }
        }
        // DuckDuckGo can return phrase objects or an OpenSearch-style list.
        "duckduckgo" => {
            if let Some(items) = json.as_array() {
                for item in items {
                    if let Some(text) = item.as_str() {
                        suggestions.insert(text.to_string());
                    } else {
                        let phrase = json_str(item, "phrase");
                        if !phrase.is_empty() {
                            suggestions.insert(phrase);
                        }
                    }
                }
            }
            if let Some(items) = json_path_array(&json, 1) {
                for item in items {
                    if let Some(text) = item.as_str() {
                        suggestions.insert(text.to_string());
                    }
                }
            }
        }
        _ => return Vec::new(),
    }
    suggestions.into_iter().take(10).collect()
}

fn json_path_array(value: &serde_json::Value, index: usize) -> Option<&[serde_json::Value]> {
    value.as_array()?.get(index)?.as_array().map(Vec::as_slice)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EngineResponse;

    fn response(body: &str) -> EngineResponse {
        EngineResponse::new(200, "https://example.com/", body.as_bytes().to_vec())
    }

    #[test]
    fn duckduckgo_suggestions_are_parsed() {
        let body = r#"[{"phrase":"rust"},{"phrase":"rust book"}]"#;
        assert_eq!(
            parse_suggestions("duckduckgo", &response(body)),
            vec!["rust".to_string(), "rust book".to_string()]
        );
    }

    #[test]
    fn duckduckgo_list_suggestions_are_parsed() {
        let body = r#"["rust",["rust","rust book","rustup"]]"#;
        assert_eq!(
            parse_suggestions("duckduckgo", &response(body)),
            vec![
                "rust".to_string(),
                "rust book".to_string(),
                "rustup".to_string()
            ]
        );
    }

    #[test]
    fn wikipedia_suggestions_are_parsed() {
        let body = r#"["rust",["Rust","Rust (programming language)"]]"#;
        assert_eq!(
            parse_suggestions("wikipedia", &response(body)),
            vec![
                "Rust".to_string(),
                "Rust (programming language)".to_string()
            ]
        );
    }

    #[test]
    fn broken_json_yields_no_suggestions() {
        assert!(parse_suggestions("duckduckgo", &response("<html>")).is_empty());
    }

    #[test]
    fn removed_and_unknown_backends_have_no_suggestions() {
        assert_eq!(BACKENDS, &["duckduckgo", "wikipedia", "mwmbl"]);
        assert_eq!(Settings::defaults().search.autocomplete, "duckduckgo");
        for backend in ["google", "unknown", "", "none"] {
            assert!(parse_suggestions(backend, &response(r#"[{"phrase":"rust"}]"#)).is_empty());
            assert!(parse_suggestions(backend, &response(r#"["rust",["rust book"]]"#)).is_empty());
        }
    }
}
