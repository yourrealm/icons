//! Routes. A mark is `/{set}/{name}.{svg|png|ico}` with rendering options in
//! the query; `/api/sets` lists what the sets contain, for the UI.

use crate::glyph::{self, Error, Set};
use crate::mark::{self, Crop, Glyph, Options, Theme};
use crate::render;
use axum::extract::{Path, Query};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::LazyLock;

pub fn router() -> Router {
    let mut r = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/api/sets", get(sets));
    // One route per set, so /assets/... and the like reach the web fallback.
    for (path, set) in [
        ("/char/{name}", Set::Char),
        ("/fa/{name}", Set::Fa),
        ("/ph/{name}", Set::Ph),
        ("/shape/{name}", Set::Shape),
    ] {
        r = r.route(
            path,
            get(move |Path(name): Path<String>, Query(q): Query<Params>| mark(set, name, q)),
        );
    }
    r.fallback(crate::web::serve)
}

/// A week: long enough that hotlinks and previews are free, short enough
/// that a change to the recipe reaches everyone without renaming URLs.
const CACHE: &str = "public, max-age=604800";

const MAX_LABEL: usize = 64;
const DEFAULT_SIZE: u32 = 512;
const MAX_SIZE: u32 = 1024;
/// Below this, grain is only noise, so PNGs drop it (and ICO always does).
const GRAIN_MIN_PX: u32 = 65;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    Svg,
    Png,
    Ico,
}

impl Format {
    fn parse(ext: &str) -> Option<Format> {
        Some(match ext {
            "svg" => Format::Svg,
            "png" => Format::Png,
            "ico" => Format::Ico,
            _ => return None,
        })
    }
    fn mime(self) -> &'static str {
        match self {
            Format::Svg => "image/svg+xml",
            Format::Png => "image/png",
            Format::Ico => "image/x-icon",
        }
    }
}

// All strings so a bad value is our 400, not axum's plain-text rejection.
#[derive(Deserialize, Default)]
struct Params {
    theme: Option<String>,
    crop: Option<String>,
    grain: Option<String>,
    label: Option<String>,
    size: Option<String>,
}

struct Request {
    options: Options,
    format: Format,
    size: u32,
}

fn parse(q: &Params, stem: &str, format: Format) -> Result<Request, String> {
    let theme = match q.theme.as_deref() {
        None | Some("light") => Theme::Light,
        Some("dark") => Theme::Dark,
        Some("auto") if format == Format::Svg => Theme::Auto,
        Some("auto") => return Err("theme=auto is only for svg".into()),
        Some(_) => return Err("theme is light, dark or auto".into()),
    };
    let crop = match q.crop.as_deref() {
        None => {
            if format == Format::Ico {
                Crop::Tile
            } else {
                Crop::Full
            }
        }
        Some("full") => Crop::Full,
        Some("tile") => Crop::Tile,
        Some(_) => return Err("crop is full or tile".into()),
    };
    let size = match (&q.size, format) {
        (None, _) => DEFAULT_SIZE,
        (Some(_), Format::Svg | Format::Ico) => return Err("size is only for png".into()),
        (Some(s), Format::Png) => match s.parse::<u32>() {
            Ok(n) if (16..=MAX_SIZE).contains(&n) => n,
            _ => return Err(format!("size is 16 to {MAX_SIZE}")),
        },
    };
    let mut grain = match &q.grain {
        None => 1.0,
        Some(s) => match s.parse::<f64>() {
            Ok(g) if (0.0..=1.0).contains(&g) => g,
            _ => return Err("grain is 0 to 1".into()),
        },
    };
    if format == Format::Ico || (format == Format::Png && size < GRAIN_MIN_PX) {
        grain = 0.0;
    }
    let label = match &q.label {
        None => stem.to_string(),
        Some(l) if l.chars().count() <= MAX_LABEL && !l.trim().is_empty() => l.clone(),
        Some(_) => return Err(format!("label is 1 to {MAX_LABEL} characters")),
    };
    Ok(Request {
        options: Options {
            theme,
            crop,
            grain,
            label,
        },
        format,
        size,
    })
}

async fn mark(set: Set, name: String, q: Params) -> Response {
    let Some((stem, ext)) = name.rsplit_once('.') else {
        return error(
            StatusCode::BAD_REQUEST,
            "name needs an extension: svg, png or ico",
        );
    };
    let Some(format) = Format::parse(ext) else {
        return error(StatusCode::BAD_REQUEST, "format is svg, png or ico");
    };
    let req = match parse(&q, stem, format) {
        Ok(r) => r,
        Err(e) => return error(StatusCode::BAD_REQUEST, &e),
    };
    let glyph = match glyph::glyph(set, stem) {
        Ok(g) => g,
        Err(Error::NotFound(m)) => return error(StatusCode::NOT_FOUND, &m),
        Err(Error::Bad(m)) => return error(StatusCode::BAD_REQUEST, &m),
    };
    match tokio::task::spawn_blocking(move || render(&glyph, &req)).await {
        Ok(Ok(bytes)) => ok(req_mime(format), bytes),
        Ok(Err(e)) => error(StatusCode::INTERNAL_SERVER_ERROR, &e),
        Err(e) => error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

fn req_mime(f: Format) -> &'static str {
    f.mime()
}

fn render(g: &Glyph, r: &Request) -> Result<Vec<u8>, String> {
    let svg = mark::svg(g, &r.options);
    match r.format {
        Format::Svg => Ok(svg.into_bytes()),
        Format::Png => render::png(&svg, r.size),
        Format::Ico => render::ico(&svg),
    }
}

fn ok(mime: &'static str, bytes: Vec<u8>) -> Response {
    let mut h = DefaultHasher::new();
    bytes.hash(&mut h);
    let etag = format!("\"{:016x}\"", h.finish());
    Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, CACHE)
        .header(header::ETAG, etag)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(bytes.into())
        .expect("static headers")
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

static SETS: LazyLock<String> = LazyLock::new(|| {
    json!({
        "char": { "font": "Nunito Black" },
        "fa": glyph::names(Set::Fa),
        "ph": glyph::names(Set::Ph),
        "shape": glyph::names(Set::Shape),
    })
    .to_string()
});

async fn sets() -> Response {
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        SETS.as_str(),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{HeaderMap, Request};
    use tower::ServiceExt;

    async fn get(uri: &str) -> (StatusCode, HeaderMap, Vec<u8>) {
        let res = router()
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = res.status();
        let headers = res.headers().clone();
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, headers, body.to_vec())
    }

    async fn text(uri: &str) -> (StatusCode, String) {
        let (s, _, b) = get(uri).await;
        (s, String::from_utf8(b).unwrap())
    }

    #[tokio::test]
    async fn character_svg() {
        let (status, headers, body) = get("/char/%C3%B7.svg").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers[header::CONTENT_TYPE], "image/svg+xml");
        assert_eq!(headers[header::ACCESS_CONTROL_ALLOW_ORIGIN], "*");
        assert!(headers.contains_key(header::ETAG));
        let svg = String::from_utf8(body).unwrap();
        assert!(svg.contains("aria-label=\"÷\""));
        assert!(svg.contains("<feTurbulence"));
    }

    #[tokio::test]
    async fn options() {
        let (_, s) = text("/shape/divide.svg?grain=0&label=expenses&theme=dark").await;
        assert!(!s.contains("<feTurbulence"));
        assert!(s.contains("<title>expenses</title>"));
        assert!(s.contains("#2c2450"));
        let (_, s) = text("/shape/ring.svg?theme=auto&crop=tile").await;
        assert!(s.contains("prefers-color-scheme"));
        assert!(s.contains("viewBox=\"32 32 336 336\""));
        let (_, s) = text("/fa/piggy-bank.svg").await;
        assert!(s.contains("Font Awesome Free"));
    }

    #[tokio::test]
    async fn png_and_ico() {
        let (status, headers, png) = get("/ph/wallet.png?size=64").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers[header::CONTENT_TYPE], "image/png");
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 64);
        let (status, headers, ico) = get("/fa/wallet.ico").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers[header::CONTENT_TYPE], "image/x-icon");
        assert_eq!(&ico[..4], &[0, 0, 1, 0]);
        assert_eq!(u16::from_le_bytes([ico[4], ico[5]]), 3);
    }

    #[tokio::test]
    async fn rejections() {
        for (uri, status) in [
            ("/fa/does-not-exist.svg", StatusCode::NOT_FOUND),
            ("/char/%E6%BC%A2.svg", StatusCode::NOT_FOUND),
            ("/char/ab.svg", StatusCode::BAD_REQUEST),
            ("/fa/wallet", StatusCode::BAD_REQUEST),
            ("/fa/wallet.gif", StatusCode::BAD_REQUEST),
            ("/fa/Wallet.svg", StatusCode::BAD_REQUEST),
            ("/fa/wallet.png?size=9999", StatusCode::BAD_REQUEST),
            ("/fa/wallet.svg?size=64", StatusCode::BAD_REQUEST),
            ("/fa/wallet.svg?grain=2", StatusCode::BAD_REQUEST),
            ("/fa/wallet.png?theme=auto", StatusCode::BAD_REQUEST),
            ("/fa/wallet.svg?theme=sepia", StatusCode::BAD_REQUEST),
        ] {
            let (s, body) = text(uri).await;
            assert_eq!(s, status, "{uri}");
            assert!(body.starts_with("{\"error\":"), "{uri}: {body}");
        }
    }

    #[tokio::test]
    async fn other_paths_reach_the_web_ui() {
        let (status, headers, _) = get("/nope/x.svg").await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            headers[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with("text/html")
        );
    }

    #[tokio::test]
    async fn sets_list_the_icons() {
        let (status, body) = text("/api/sets").await;
        assert_eq!(status, StatusCode::OK);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(
            v["fa"]
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n == "piggy-bank")
        );
        assert!(v["ph"].as_array().unwrap().iter().any(|n| n == "wallet"));
        assert_eq!(v["shape"], json!(["divide", "ring"]));
    }
}
