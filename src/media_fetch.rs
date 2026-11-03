// SPDX-License-Identifier: AGPL-3.0-or-later
//! Direct-only media destination validation; no page reading or HTML extraction.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use reqwest::dns::{Addrs, Name, Resolve, Resolving};

const FETCH_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_REDIRECTS: usize = 5;

/// Whether an address is globally routable, including embedded IPv4 addresses.
fn is_global(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => is_global_v4(address),
        IpAddr::V6(address) => is_global_v6(address),
    }
}

fn is_global_v4(address: Ipv4Addr) -> bool {
    let [a, b, c, _] = address.octets();
    !(a == 0
        || a == 10
        || a == 127
        || a >= 224
        || (a == 100 && (64..128).contains(&b))
        || (a == 169 && b == 254)
        || (a == 172 && (16..32).contains(&b))
        || (a == 192 && b == 168)
        || (a == 192 && b == 0 && c == 0)
        || (a == 192 && b == 0 && c == 2)
        || (a == 198 && (18..20).contains(&b))
        || (a == 198 && b == 51 && c == 100)
        || (a == 203 && b == 0 && c == 113))
}

fn is_global_v6(address: Ipv6Addr) -> bool {
    if let Some(v4) = address.to_ipv4_mapped() {
        return is_global_v4(v4);
    }
    let segments = address.segments();
    let embedded_v4 = |high: u16, low: u16| {
        Ipv4Addr::new((high >> 8) as u8, high as u8, (low >> 8) as u8, low as u8)
    };
    match segments[0] {
        0x0064 if segments[1] == 0xff9b => is_global_v4(embedded_v4(segments[6], segments[7])),
        0x2002 => is_global_v4(embedded_v4(segments[1], segments[2])),
        0x2001 => segments[1] >= 0x0200 && segments[1] != 0x0db8,
        first => (0x2000..0x4000).contains(&first) && first != 0x3fff,
    }
}

/// Allow only HTTP(S) media URLs; hostnames are checked again by the resolver.
pub(crate) fn is_public_media_url(url: &reqwest::Url) -> bool {
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    match url.host() {
        Some(url::Host::Ipv4(address)) => is_global_v4(address),
        Some(url::Host::Ipv6(address)) => is_global_v6(address),
        Some(url::Host::Domain(domain)) => {
            let domain = domain.trim_end_matches('.').to_ascii_lowercase();
            domain != "localhost" && !domain.ends_with(".localhost")
        }
        None => false,
    }
}

struct PublicResolver;

impl Resolve for PublicResolver {
    fn resolve(&self, name: Name) -> Resolving {
        Box::pin(async move {
            let host = name.as_str().to_string();
            let addresses: Vec<SocketAddr> = tokio::net::lookup_host((host.as_str(), 0))
                .await?
                .filter(|address| is_global(address.ip()))
                .collect();
            if addresses.is_empty() {
                return Err(format!("{host} has no public address").into());
            }
            Ok(Box::new(addresses.into_iter()) as Addrs)
        })
    }
}

/// Build a media-only client; engine endpoints never grant private-host exceptions.
/// Address overrides are reserved for local test fixtures.
pub(crate) fn public_media_client(
    outgoing: &crate::settings::Outgoing,
    overrides: &[(String, SocketAddr)],
) -> Result<(reqwest::Client, bool), String> {
    let redirects = reqwest::redirect::Policy::custom(|attempt| {
        if attempt.previous().len() >= MAX_REDIRECTS {
            attempt.error("too many redirects")
        } else if is_public_media_url(attempt.url()) {
            attempt.follow()
        } else {
            attempt.error("redirect to a non-public address")
        }
    });
    let mut builder = reqwest::Client::builder()
        .no_proxy()
        .redirect(redirects)
        .dns_resolver(Arc::new(PublicResolver))
        .timeout(FETCH_TIMEOUT)
        .connect_timeout(Duration::from_secs(4))
        .danger_accept_invalid_certs(!outgoing.verify)
        .user_agent(crate::network::default_user_agent());
    if !outgoing.enable_http2 {
        builder = builder.http1_only();
    }
    let proxy_blocked =
        !outgoing.proxies.is_empty() || outgoing.using_tor_proxy || environment_proxy_configured();
    for (host, address) in overrides {
        builder = builder.resolve(host, *address);
    }
    Ok((
        builder.build().map_err(|error| error.to_string())?,
        proxy_blocked,
    ))
}

fn environment_proxy_configured() -> bool {
    [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
    ]
    .iter()
    .any(|name| {
        std::env::var(name)
            .ok()
            .is_some_and(|value| !value.trim().is_empty())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_global_addresses_are_public() {
        for private in [
            "0.0.0.0",
            "10.1.2.3",
            "100.64.0.1",
            "127.0.0.1",
            "169.254.169.254",
            "172.16.0.1",
            "192.0.0.8",
            "192.0.2.1",
            "192.168.1.1",
            "198.18.0.1",
            "224.0.0.1",
            "255.255.255.255",
            "::",
            "::1",
            "::127.0.0.1",
            "::ffff:127.0.0.1",
            "::ffff:10.0.0.1",
            "64:ff9b::a9fe:a9fe",
            "2002:7f00:1::",
            "2001::1",
            "2001:db8::1",
            "fc00::1",
            "fd12:3456::1",
            "fe80::1",
            "ff02::1",
            "100::1",
        ] {
            assert!(!is_global(private.parse().unwrap()), "{private}");
        }
        for public in [
            "1.1.1.1",
            "8.8.8.8",
            "93.184.215.14",
            "::ffff:8.8.8.8",
            "64:ff9b::808:808",
            "2002:808:808::",
            "2606:4700::1111",
            "2a00:1450::1",
            "2001:4860::8888",
        ] {
            assert!(is_global(public.parse().unwrap()), "{public}");
        }
    }

    #[test]
    fn literal_hosts_and_schemes_are_checked() {
        for refused in [
            "http://127.0.0.1/",
            "http://[::ffff:127.0.0.1]:8080/",
            "http://[fe80::1]/",
            "http://localhost/",
            "http://a.localhost/",
            "http://LOCALHOST./",
            "file:///etc/passwd",
            "ftp://example.com/",
            "http://169.254.169.254/latest",
        ] {
            assert!(
                !is_public_media_url(&reqwest::Url::parse(refused).unwrap()),
                "{refused}"
            );
        }
        for allowed in [
            "https://example.com/",
            "http://8.8.8.8/",
            "https://[2606:4700::1111]/",
        ] {
            assert!(
                is_public_media_url(&reqwest::Url::parse(allowed).unwrap()),
                "{allowed}"
            );
        }
    }

    #[test]
    fn configured_proxies_fail_closed() {
        let mut outgoing = crate::settings::Outgoing::default();
        outgoing
            .proxies
            .insert("all".into(), "http://127.0.0.1:8080".into());
        assert!(public_media_client(&outgoing, &[]).unwrap().1);
    }

    #[tokio::test]
    async fn names_resolving_to_private_addresses_are_refused() {
        let error = PublicResolver
            .resolve("localhost".parse().unwrap())
            .await
            .err()
            .expect("localhost must not resolve");
        assert!(error.to_string().contains("no public address"), "{error}");
    }
}
