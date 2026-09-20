//! Where glyphs come from: a character of Nunito Black, an icon from one of
//! the vendored sets, or a built-in shape. Each becomes a `mark::Glyph`.
//!
//! The sets live in ../assets, put there by scripts/vendor.sh, and are
//! embedded in the binary with their licenses. Font Awesome's icons are
//! CC BY 4.0: the comment in each file is the attribution, and it is carried
//! into every mark made from one.

use crate::mark::Glyph;
use resvg::usvg;
use rust_embed::Embed;
use std::fmt::Write;
use ttf_parser::{Face, OutlineBuilder, Tag};

#[derive(Embed)]
#[folder = "assets/fa"]
#[include = "*.svg"]
struct Fa;

#[derive(Embed)]
#[folder = "assets/ph"]
#[include = "*.svg"]
struct Ph;

static FONT: &[u8] = include_bytes!("../assets/font/Nunito[wght].ttf");

pub const SHAPES: [&str; 2] = ["divide", "ring"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Set {
    Char,
    Fa,
    Ph,
    Shape,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    NotFound(String),
    Bad(String),
}

pub fn glyph(set: Set, name: &str) -> Result<Glyph, Error> {
    match set {
        Set::Char => char_glyph(name),
        Set::Fa => icon(name, Fa::get),
        Set::Ph => icon(name, Ph::get),
        Set::Shape => shape(name),
    }
}

/// Icon names in a set, sorted. Empty for `char`.
pub fn names(set: Set) -> Vec<String> {
    let mut v: Vec<String> = match set {
        Set::Char => Vec::new(),
        Set::Fa => Fa::iter().filter_map(stem).collect(),
        Set::Ph => Ph::iter().filter_map(stem).collect(),
        Set::Shape => SHAPES.iter().map(|s| s.to_string()).collect(),
    };
    v.sort();
    v
}

fn stem(path: std::borrow::Cow<'static, str>) -> Option<String> {
    path.strip_suffix(".svg").map(String::from)
}

// ---- characters -------------------------------------------------------------

#[derive(Default)]
struct PathBuilder {
    d: String,
}

// Font y grows upward; SVG y grows downward.
impl OutlineBuilder for PathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        write!(self.d, "M{x:.1} {:.1} ", -y).unwrap();
    }
    fn line_to(&mut self, x: f32, y: f32) {
        write!(self.d, "L{x:.1} {:.1} ", -y).unwrap();
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        write!(self.d, "Q{x1:.1} {:.1} {x:.1} {:.1} ", -y1, -y).unwrap();
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        write!(
            self.d,
            "C{x1:.1} {:.1} {x2:.1} {:.1} {x:.1} {:.1} ",
            -y1, -y2, -y
        )
        .unwrap();
    }
    fn close(&mut self) {
        self.d.push_str("Z ");
    }
}

fn char_glyph(name: &str) -> Result<Glyph, Error> {
    let mut chars = name.chars();
    let (Some(c), None) = (chars.next(), chars.next()) else {
        return Err(Error::Bad("exactly one character".into()));
    };
    let mut face = Face::parse(FONT, 0).map_err(|e| Error::Bad(format!("font: {e}")))?;
    face.set_variation(Tag::from_bytes(b"wght"), 900.0);
    let id = face
        .glyph_index(c)
        .ok_or_else(|| Error::NotFound(format!("the font has no {c:?}")))?;
    let mut b = PathBuilder::default();
    let r = face
        .outline_glyph(id, &mut b)
        .ok_or_else(|| Error::NotFound(format!("{c:?} has no outline")))?;
    Ok(Glyph {
        body: format!("<path d=\"{}\"/>", b.d.trim_end()),
        x: r.x_min as f64,
        y: -(r.y_max as f64),
        w: (r.x_max - r.x_min) as f64,
        h: (r.y_max - r.y_min) as f64,
        placed: false,
        notice: None,
    })
}

// ---- icon sets --------------------------------------------------------------

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn icon(name: &str, get: fn(&str) -> Option<rust_embed::EmbeddedFile>) -> Result<Glyph, Error> {
    if !valid_name(name) {
        return Err(Error::Bad(
            "icon names are lowercase, digits and dashes".into(),
        ));
    }
    let file =
        get(&format!("{name}.svg")).ok_or_else(|| Error::NotFound(format!("no icon {name}")))?;
    let text = std::str::from_utf8(&file.data).map_err(|e| Error::Bad(e.to_string()))?;
    icon_glyph(text)
}

/// An icon file is one `<svg>` with a viewBox and paths inside, and for Font
/// Awesome a license comment first. The ink box comes from usvg.
fn icon_glyph(text: &str) -> Result<Glyph, Error> {
    let bad = |m: &str| Error::Bad(format!("icon: {m}"));
    let open_end = text.find('>').ok_or_else(|| bad("no opening tag"))? + 1;
    let close = text.rfind("</svg>").ok_or_else(|| bad("no closing tag"))?;
    let mut inner = text[open_end..close].trim();
    let mut notice = None;
    if let Some(rest) = inner.strip_prefix("<!--") {
        let end = rest
            .find("-->")
            .ok_or_else(|| bad("unterminated comment"))?;
        notice = Some(format!("<!--{}-->", &rest[..end]));
        inner = rest[end + 3..].trim();
    }
    // usvg reports the box after the viewBox transform; add the origin back so
    // it is in the same coordinates as the paths we embed.
    let vb = text
        .split_once("viewBox=\"")
        .and_then(|(_, r)| r.split_once('"'))
        .map(|(v, _)| v)
        .ok_or_else(|| bad("no viewBox"))?;
    let origin: Vec<f64> = vb
        .split_whitespace()
        .take(2)
        .filter_map(|n| n.parse().ok())
        .collect();
    let [ox, oy] = origin[..] else {
        return Err(bad("bad viewBox"));
    };
    let tree =
        usvg::Tree::from_str(text, &usvg::Options::default()).map_err(|e| bad(&e.to_string()))?;
    let r = tree.root().abs_bounding_box();
    Ok(Glyph {
        body: inner.to_string(),
        x: r.x() as f64 + ox,
        y: r.y() as f64 + oy,
        w: r.width() as f64,
        h: r.height() as f64,
        placed: false,
        notice,
    })
}

// ---- shapes -----------------------------------------------------------------

/// The marks expenses and loans ship, as drawn in their generators: ink spans
/// 104..296 in mark space, bars 52 thick.
fn shape(name: &str) -> Result<Glyph, Error> {
    let d = match name {
        "divide" => divide(),
        "ring" => ring(),
        _ => return Err(Error::NotFound(format!("no shape {name}"))),
    };
    Ok(Glyph {
        body: format!("<path d=\"{d}\"/>"),
        x: 104.0,
        y: 104.0,
        w: 192.0,
        h: 192.0,
        placed: true,
        notice: None,
    })
}

/// A stadium and two discs: two people and a split.
fn divide() -> String {
    let disc = |cx: f64, cy: f64, r: f64| {
        format!(
            "M {} {cy} A {r} {r} 0 1 0 {} {cy} A {r} {r} 0 1 0 {} {cy} Z",
            cx - r,
            cx + r,
            cx - r
        )
    };
    format!(
        "M 130 174 L 270 174 A 26 26 0 0 1 270 226 L 130 226 A 26 26 0 0 1 130 174 Z {} {}",
        disc(200.0, 132.0, 28.0),
        disc(200.0, 268.0, 28.0)
    )
}

/// A ring open at the top with round ends: a loan being paid down. Every
/// subpath winds the same way so the caps add to the ring under nonzero fill.
fn ring() -> String {
    const OUTER: f64 = 96.0;
    const HALF: f64 = 26.0;
    const GAP: f64 = 34.0; // degrees either side of the top
    let mean = OUTER - HALF;
    let inner = OUTER - 2.0 * HALF;
    let at = |deg: f64, r: f64| {
        let t = deg.to_radians();
        (200.0 + r * t.cos(), 200.0 + r * t.sin())
    };
    let pt = |(x, y): (f64, f64)| format!("{x:.2} {y:.2}");
    let disc = |(cx, cy): (f64, f64), r: f64| {
        format!(
            "M {} A {r} {r} 0 1 1 {} A {r} {r} 0 1 1 {} Z",
            pt((cx - r, cy)),
            pt((cx + r, cy)),
            pt((cx - r, cy))
        )
    };
    let start = -90.0 + GAP;
    let end = 270.0 - GAP;
    format!(
        "M {} A {OUTER} {OUTER} 0 1 1 {} L {} A {inner} {inner} 0 1 0 {} Z {} {}",
        pt(at(start, OUTER)),
        pt(at(end, OUTER)),
        pt(at(end, inner)),
        pt(at(start, inner)),
        disc(at(start, mean), HALF),
        disc(at(end, mean), HALF)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn characters_come_from_the_font() {
        let g = glyph(Set::Char, "÷").unwrap();
        assert!(g.body.starts_with("<path d=\"M"));
        assert!(g.w > 0.0 && g.h > 0.0);
        assert_eq!(
            glyph(Set::Char, "ab").unwrap_err(),
            Error::Bad("exactly one character".into())
        );
        assert!(matches!(glyph(Set::Char, "漢"), Err(Error::NotFound(_))));
    }

    #[test]
    fn icons_keep_their_notice_and_lose_their_wrapper() {
        let g = glyph(Set::Fa, "piggy-bank").unwrap();
        assert!(g.notice.as_deref().unwrap().contains("Font Awesome Free"));
        assert!(g.body.starts_with("<path"));
        assert!(!g.body.contains("<svg"));
        let p = glyph(Set::Ph, "wallet").unwrap();
        assert_eq!(p.notice, None);
        assert!(p.w > 0.0);
    }

    #[test]
    fn names_are_checked() {
        assert!(matches!(glyph(Set::Fa, "Wallet"), Err(Error::Bad(_))));
        assert!(matches!(glyph(Set::Fa, "../x"), Err(Error::Bad(_))));
        assert!(matches!(
            glyph(Set::Ph, "no-such-icon"),
            Err(Error::NotFound(_))
        ));
        assert!(names(Set::Fa).contains(&"piggy-bank".to_string()));
        assert!(names(Set::Ph).len() > 1000);
    }

    #[test]
    fn shapes_are_placed() {
        assert!(glyph(Set::Shape, "divide").unwrap().placed);
        assert!(glyph(Set::Shape, "ring").unwrap().body.contains("A 96 96"));
    }
}
