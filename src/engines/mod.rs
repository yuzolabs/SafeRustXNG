//! Non-HTML engine implementations, one module per upstream engine.
//!
//! Human-facing HTML scrapers and the generic XPath engine are not registered.

pub mod apple_app_store;
pub mod apple_maps;
pub mod artic;
pub mod arxiv;
pub mod astrophysics_data_system;
pub mod azure;
pub mod baidu;
pub mod boardreader;
pub mod bpb;
pub mod braveapi;
pub mod bt4g;
pub mod cachy_os;
pub mod ccc_media;
pub mod chatnoir;
pub mod chinaso;
pub mod cloudflareai;
pub mod core;
pub mod crates;
pub mod crossref;
pub mod currency_convert;
pub mod dailymotion;
pub mod deepl;
pub mod deezer;
pub mod devicons;
pub mod discourse;
pub mod docker_hub;
pub mod duckduckgo_definitions;
pub mod duckduckgo_extra;
pub mod duckduckgo_weather;
pub mod duckduckgo_web;
pub mod elasticsearch;
pub mod europepmc;
pub mod exaapi;
pub mod findthatmeme;
pub mod fireball;
pub mod flickr;
pub mod freesound;
pub mod frinkiac;
pub mod fyyd;
pub mod giphy;
pub mod gitea;
pub mod github;
pub mod github_code;
pub mod gitlab;
pub mod gmx;
pub mod hackernews;
pub mod hex;
pub mod huggingface;
pub mod iconify;
pub mod il_post;
pub mod iseek;
pub mod jina;
pub mod jisho;
pub mod json_engine;
pub mod kagi;
pub mod keenable;
pub mod lemmy;
pub mod libretranslate;
pub mod lingva;
pub mod lucide;
pub mod marginalia;
#[cfg(feature = "db-engines")]
pub mod mariadb_server;
pub mod mastodon;
pub mod material_icons;
pub mod mediathekviewweb;
pub mod mediawiki;
pub mod meilisearch;
pub mod metacpan;
pub mod microsoft_learn;
pub mod mixcloud;
#[cfg(feature = "db-engines")]
pub mod mongodb;
pub mod moviepilot;
pub mod mozhi;
pub mod mwmbl;
#[cfg(feature = "db-engines")]
pub mod mysql_server;
pub mod neosearch;
pub mod nine_gag;
pub mod npm;
pub mod nvd;
pub mod odysee;
pub mod open_meteo;
pub mod openalex;
pub mod openlibrary;
pub mod opensemantic;
pub mod openstreetmap;
pub mod openverse;
pub mod pdbe;
pub mod peertube;
pub mod photon;
pub mod piratebay;
pub mod podchaser;
#[cfg(feature = "db-engines")]
pub mod postgresql;
pub mod public_domain_image_archive;
pub mod pubmed;
pub mod r360search_videos;
pub mod radio_browser;
pub mod recoll;
pub mod repology;
pub mod reuters;
pub mod scanr_structures;
pub mod searchzee;
pub mod seekninja;
pub mod selfhst;
pub mod semantic_scholar;
pub mod senscritique;
pub mod sepiasearch;
pub mod solr;
pub mod spotify;
pub mod springer;
#[cfg(feature = "db-engines")]
pub mod sqlite;
pub mod stackexchange;
pub mod startpagina;
pub mod steam;
pub mod swisscows;
pub mod swisscows_news;
pub mod tagesschau;
pub mod tineye;
pub mod tootfinder;
pub mod torznab;
pub mod translated;
pub mod tusksearch;
#[cfg(feature = "db-engines")]
pub mod valkey_server;
pub mod vimeo;
pub mod voidlinux;
pub mod wallhaven;
pub mod wikicommons;
pub mod wikidata;
pub mod wikipedia;
pub mod wolframalpha_api;
pub mod wttr;
pub mod xprivo;
pub mod yacy;
pub mod yandex_api;
pub mod yandex_music;
pub mod youtube_api;

use crate::engine::EngineHandle;
use crate::settings::EngineConfig;

/// Build the engine implementation selected by `config.engine`.
pub fn build(config: &EngineConfig) -> Result<EngineHandle, String> {
    let handle: EngineHandle = match config.engine.as_str() {
        "arxiv" => arxiv::Arxiv::from_config(config)?,
        "crates" => crates::Crates::from_config(config)?,
        "crossref" => crossref::Crossref::from_config(config)?,
        "dailymotion" => dailymotion::Dailymotion::from_config(config)?,
        "deezer" => deezer::Deezer::from_config(config)?,
        "docker_hub" => docker_hub::DockerHub::from_config(config)?,
        "github" => github::GitHub::from_config(config)?,
        "hackernews" => hackernews::HackerNews::from_config(config)?,
        "mwmbl" => mwmbl::Mwmbl::from_config(config)?,
        "npm" => npm::Npm::from_config(config)?,
        "openlibrary" => openlibrary::OpenLibrary::from_config(config)?,
        "openstreetmap" => openstreetmap::Openstreetmap::from_config(config)?,
        "openverse" => openverse::Openverse::from_config(config)?,
        "photon" => photon::Photon::from_config(config)?,
        "semantic_scholar" => semantic_scholar::SemanticScholar::from_config(config)?,
        "stackexchange" => stackexchange::StackExchange::from_config(config)?,
        "wikicommons" => wikicommons::Wikicommons::from_config(config)?,
        "wikidata" => wikidata::Wikidata::from_config(config)?,
        "wikipedia" => wikipedia::Wikipedia::from_config(config)?,
        "json_engine" => json_engine::JsonEngine::from_config(config)?,
        "mediawiki" => mediawiki::MediaWiki::from_config(config)?,
        "apple_app_store" => apple_app_store::AppleAppStore::from_config(config)?,
        "baidu" => baidu::Baidu::from_config(config)?,
        "fireball" => fireball::Fireball::from_config(config)?,
        "lemmy" => lemmy::Lemmy::from_config(config)?,
        "mastodon" => mastodon::Mastodon::from_config(config)?,
        "9gag" => nine_gag::NineGag::from_config(config)?,
        "peertube" => peertube::Peertube::from_config(config)?,
        "sepiasearch" => sepiasearch::Sepiasearch::from_config(config)?,
        "startpagina" => startpagina::Startpagina::from_config(config)?,
        "tootfinder" => tootfinder::Tootfinder::from_config(config)?,
        "tusksearch" => tusksearch::Tusksearch::from_config(config)?,
        "mixcloud" => mixcloud::Mixcloud::from_config(config)?,
        "wallhaven" => wallhaven::Wallhaven::from_config(config)?,
        "giphy" => giphy::Giphy::from_config(config)?,
        "discourse" => discourse::Discourse::from_config(config)?,
        "duckduckgo_extra" => duckduckgo_extra::DuckDuckGoExtra::from_config(config)?,
        "duckduckgo_web" => duckduckgo_web::DuckDuckGoWeb::from_config(config)?,
        "artic" => artic::Artic::from_config(config)?,
        "bpb" => bpb::Bpb::from_config(config)?,
        "braveapi" => braveapi::BraveApi::from_config(config)?,
        "cachy_os" => cachy_os::CachyOs::from_config(config)?,
        "ccc_media" => ccc_media::CccMedia::from_config(config)?,
        "chinaso" => chinaso::ChinaSo::from_config(config)?,
        "cloudflareai" => cloudflareai::CloudflareAi::from_config(config)?,
        "core" => core::Core::from_config(config)?,
        "360search_videos" => r360search_videos::Videoscarch360::from_config(config)?,
        "odysee" => odysee::Odysee::from_config(config)?,
        "vimeo" => vimeo::Vimeo::from_config(config)?,
        "youtube_api" => youtube_api::YouTubeApi::from_config(config)?,
        "github_code" => github_code::GitHubCode::from_config(config)?,
        "gitlab" => gitlab::GitLab::from_config(config)?,
        "marginalia" => marginalia::Marginalia::from_config(config)?,
        "europepmc" => europepmc::EuropePmc::from_config(config)?,
        "gitea" => gitea::Gitea::from_config(config)?,
        "hex" => hex::Hex::from_config(config)?,
        "neosearch" => neosearch::Neosearch::from_config(config)?,
        "openalex" => openalex::OpenAlex::from_config(config)?,
        "springer" => springer::Springer::from_config(config)?,
        "steam" => steam::Steam::from_config(config)?,
        "swisscows" => swisscows::Swisscows::from_config(config)?,
        "flickr" => flickr::Flickr::from_config(config)?,
        "iconify" => iconify::Iconify::from_config(config)?,
        "jisho" => jisho::Jisho::from_config(config)?,
        "pubmed" => pubmed::PubMed::from_config(config)?,
        "repology" => repology::Repology::from_config(config)?,
        "wolframalpha_api" => wolframalpha_api::WolframAlphaApi::from_config(config)?,
        "currency_convert" => currency_convert::CurrencyConvert::from_config(config)?,
        "open_meteo" => open_meteo::OpenMeteo::from_config(config)?,
        "piratebay" => piratebay::Piratebay::from_config(config)?,
        "bt4g" => bt4g::Bt4g::from_config(config)?,
        "huggingface" => huggingface::HuggingFace::from_config(config)?,
        "jina" => jina::Jina::from_config(config)?,
        "metacpan" => metacpan::Metacpan::from_config(config)?,
        "nvd" => nvd::Nvd::from_config(config)?,
        "pdbe" => pdbe::Pdbe::from_config(config)?,
        "torznab" => torznab::Torznab::from_config(config)?,
        "apple_maps" => apple_maps::AppleMaps::from_config(config)?,
        "boardreader" => boardreader::Boardreader::from_config(config)?,
        "devicons" => devicons::Devicons::from_config(config)?,
        "duckduckgo_definitions" => {
            duckduckgo_definitions::DuckDuckGoDefinitions::from_config(config)?
        }
        "il_post" => il_post::IlPost::from_config(config)?,
        "lingva" => lingva::Lingva::from_config(config)?,
        "mediathekviewweb" => mediathekviewweb::Mediathekviewweb::from_config(config)?,
        "microsoft_learn" => microsoft_learn::MicrosoftLearn::from_config(config)?,
        "moviepilot" => moviepilot::Moviepilot::from_config(config)?,
        "podchaser" => podchaser::Podchaser::from_config(config)?,
        "reuters" => reuters::Reuters::from_config(config)?,
        "scanr_structures" => scanr_structures::ScanrStructures::from_config(config)?,
        "iseek" => iseek::Iseek::from_config(config)?,
        "keenable" => keenable::Keenable::from_config(config)?,
        "libretranslate" => libretranslate::LibreTranslate::from_config(config)?,
        "mozhi" => mozhi::Mozhi::from_config(config)?,
        "public_domain_image_archive" => {
            public_domain_image_archive::PublicDomainImageArchive::from_config(config)?
        }
        "radio_browser" => radio_browser::RadioBrowser::from_config(config)?,
        "astrophysics_data_system" => {
            astrophysics_data_system::AstrophysicsDataSystem::from_config(config)?
        }
        "chatnoir" => chatnoir::Chatnoir::from_config(config)?,
        "findthatmeme" => findthatmeme::FindThatMeme::from_config(config)?,
        "frinkiac" => frinkiac::Frinkiac::from_config(config)?,
        "fyyd" => fyyd::Fyyd::from_config(config)?,
        "gmx" => gmx::Gmx::from_config(config)?,
        "lucide" => lucide::Lucide::from_config(config)?,
        "tineye" => tineye::Tineye::from_config(config)?,
        "deepl" => deepl::Deepl::from_config(config)?,
        "material_icons" => material_icons::MaterialIcons::from_config(config)?,
        "searchzee" => searchzee::SearchZee::from_config(config)?,
        "seekninja" => seekninja::SeekNinja::from_config(config)?,
        "selfhst" => selfhst::Selfhst::from_config(config)?,
        "senscritique" => senscritique::Senscritique::from_config(config)?,
        "spotify" => spotify::Spotify::from_config(config)?,
        "swisscows_news" => swisscows_news::SwisscowsNews::from_config(config)?,
        "tagesschau" => tagesschau::Tagesschau::from_config(config)?,
        "voidlinux" => voidlinux::VoidLinux::from_config(config)?,
        "xprivo" => xprivo::Xprivo::from_config(config)?,
        "exaapi" => exaapi::ExaApi::from_config(config)?,
        "freesound" => freesound::Freesound::from_config(config)?,
        "kagi" => kagi::Kagi::from_config(config)?,
        "translated" => translated::Translated::from_config(config)?,
        "wttr" => wttr::Wttr::from_config(config)?,
        "yacy" => yacy::Yacy::from_config(config)?,
        "azure" => azure::Azure::from_config(config)?,
        "meilisearch" => meilisearch::Meilisearch::from_config(config)?,
        "solr" => solr::Solr::from_config(config)?,
        "elasticsearch" => elasticsearch::Elasticsearch::from_config(config)?,
        "yandex_api" => yandex_api::YandexApi::from_config(config)?,
        "duckduckgo_weather" => duckduckgo_weather::DuckDuckGoWeather::from_config(config)?,
        "yandex_music" => yandex_music::YandexMusic::from_config(config)?,
        #[cfg(feature = "db-engines")]
        "mariadb_server" => mariadb_server::MariadbServer::from_config(config)?,
        #[cfg(feature = "db-engines")]
        "mongodb" => mongodb::Mongodb::from_config(config)?,
        #[cfg(feature = "db-engines")]
        "mysql_server" => mysql_server::MysqlServer::from_config(config)?,
        "opensemantic" => opensemantic::OpenSemantic::from_config(config)?,
        #[cfg(feature = "db-engines")]
        "postgresql" => postgresql::Postgresql::from_config(config)?,
        "recoll" => recoll::Recoll::from_config(config)?,
        #[cfg(feature = "db-engines")]
        "sqlite" => sqlite::Sqlite::from_config(config)?,
        #[cfg(feature = "db-engines")]
        "valkey_server" => valkey_server::ValkeyServer::from_config(config)?,
        other => {
            return Err(format!(
                "engine {:?} uses the unknown engine type {other:?}",
                config.name
            ));
        }
    };
    Ok(handle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    #[test]
    fn every_default_engine_builds() {
        let settings = Settings::defaults();
        for config in settings.enabled_engines() {
            build(config).unwrap_or_else(|error| {
                panic!("engine {:?} could not be built: {error}", config.name)
            });
        }
    }

    #[test]
    fn html_engines_cannot_be_restored_by_custom_settings() {
        for engine in [
            "duckduckgo",
            "google",
            "bing",
            "brave",
            "mojeek",
            "pypi",
            "youtube_noapi",
            "flickr_noapi",
            "quark",
            "startpage",
            "sogou_images",
            "s1search_rampjs",
            "xpath",
        ] {
            let config = EngineConfig::new("removed HTML engine", engine, "", &["general"]);
            assert!(
                build(&config).is_err(),
                "HTML engine {engine} is still available"
            );
        }
    }

    #[test]
    fn tos_prohibited_engines_cannot_be_restored_by_custom_settings() {
        let mut settings = Settings::defaults();
        for engine in [
            "unsplash",
            "pinterest",
            "genius",
            "imdb",
            "bitchute",
            "500px",
            "pexels",
            "qwant",
            "artstation",
            "pixiv",
            "iqiyi",
            "sogou_videos",
            "flaticon",
            "yep",
            "dogpile",
            "findborg",
            "andisearch",
            "grokipedia",
            "google_news",
            "bing_news",
            "piped",
            "invidious",
            "google_cse",
            "soundcloud",
            "wolframalpha_noapi",
            "magnific",
            "chefkoch",
            "bilibili",
            "pixabay",
            "stocksnap",
        ] {
            assert!(
                settings
                    .engines
                    .iter()
                    .all(|config| config.engine != engine),
                "removed engine {engine} is still in default settings"
            );
            let config = EngineConfig::new(engine, engine, engine, &["general"]);
            assert!(
                build(&config).is_err(),
                "removed engine {engine} is still available"
            );
            settings.engines.push(config);
        }
        // Legacy settings must not prevent the remaining engines from starting.
        let registry = crate::engine::EngineRegistry::from_settings(&settings).unwrap();
        assert_eq!(registry.len(), settings.enabled_engines().count() - 30);
        assert_eq!(registry.failed().len(), 30);
        for (name, error) in registry.failed() {
            assert!(registry.get(name).is_none());
            assert!(registry.by_shortcut(name).is_none());
            assert!(error.contains("unknown engine type"));
        }
        assert!(registry.get("wikipedia").is_some());
    }

    #[test]
    fn unknown_engine_types_are_rejected() {
        let config = EngineConfig::new("nope", "not_an_engine", "np", &["general"]);
        assert!(build(&config).is_err());
    }
}
