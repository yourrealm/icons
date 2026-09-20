//! The mark: Realm's clay tile with a glyph raised on it, as one SVG string.
//! Ported from expenses' scripts/logo/gen.ts (itself Realm's mark with the R
//! swapped out), plus a grain stage. Pure: no I/O, no rasterising.

use std::fmt::Write;
use std::sync::LazyLock;

/// Something to raise on the tile.
#[derive(Debug)]
pub struct Glyph {
    /// Inner SVG markup in the glyph's own coordinates. Fills are inherited
    /// from the group the mark wraps it in; `currentColor` is rewritten.
    pub body: String,
    /// Ink box in those coordinates.
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    /// Already drawn in mark space (the built-in shapes): not fitted.
    pub placed: bool,
    /// A license notice carried into the output verbatim.
    pub notice: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Light,
    Dark,
    /// Both marks in one SVG; `prefers-color-scheme` picks.
    Auto,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Crop {
    /// The 400-unit mark space, shadow margin included.
    Full,
    /// The tile fills the frame, for icon sizes.
    Tile,
}

pub struct Options {
    pub theme: Theme,
    pub crop: Crop,
    /// 0 for none, 1 for the default heavy grain.
    pub grain: f64,
    pub label: String,
}

struct Mode {
    tile: &'static str,
    mark: &'static str,
    dark: bool,
}

const LIGHT: Mode = Mode {
    tile: "#b4a3f2",
    mark: "#fdf3e4",
    dark: false,
};
const DARK: Mode = Mode {
    tile: "#2c2450",
    mark: "#b4a3f2",
    dark: true,
};

/// The glyph's longer side, in mark units. The tile is 320; Realm's R is
/// 168 tall, expenses' division sign spans 192.
const INK: f64 = 184.0;
const FULL_BOX: &str = "0 0 400 400";
const TILE_BOX: &str = "32 32 336 336";

// ---- colors -----------------------------------------------------------------

fn channel(hex: &str, i: usize) -> f64 {
    u8::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).expect("hex colour") as f64
}

fn mix(from: &str, to: &str, t: f64) -> String {
    let c = |i| (channel(from, i) + (channel(to, i) - channel(from, i)) * t).round() as u8;
    format!("#{:02x}{:02x}{:02x}", c(0), c(1), c(2))
}

fn tint(h: &str) -> String {
    mix(h, "#ffffff", 0.32)
}

fn shade(h: &str) -> String {
    mix(h, "#1a1030", 0.55)
}

// ---- geometry ---------------------------------------------------------------

/// Superellipse centred at 200,200; half-size `a`, exponent `n`.
fn squircle(a: f64, n: f64, steps: usize) -> String {
    let pts: Vec<String> = (0..steps)
        .map(|i| {
            let t = i as f64 / steps as f64 * std::f64::consts::TAU;
            let (c, s) = (t.cos(), t.sin());
            let x = 200.0 + a * c.signum() * c.abs().powf(2.0 / n);
            let y = 200.0 + a * s.signum() * s.abs().powf(2.0 / n);
            format!("{x:.1} {y:.1}")
        })
        .collect();
    format!("M {} Z", pts.join(" L "))
}

static TILE_PATH: LazyLock<String> = LazyLock::new(|| squircle(160.0, 4.5, 240)); // spans 40..360

// ---- clay -------------------------------------------------------------------

struct Clay<'a> {
    id: String,
    d: i32,
    shadow: &'a str,
    light_op: f64,
    dark_op: f64,
    drop_op: f64,
    /// Grain amount, 0 for none.
    grain: f64,
}

/// Drop shadow, a light inner rim top-left, a dark inner rim bottom-right, a
/// thin specular edge, then grain: fractal noise, desaturated, stretched 4x
/// around mid-grey (raw fractal noise sits near 0.5, and overlaying mid-grey
/// changes nothing), clipped to the shape and overlaid on the clay.
fn clay(c: &Clay) -> String {
    let d = c.d;
    let blur = (d as f64 * 1.5).round();
    let dy = (d as f64 * 1.75).round();
    let mut f = String::new();
    write!(
        f,
        r##"<filter id="{}" x="-45%" y="-45%" width="190%" height="190%" color-interpolation-filters="sRGB">
<feGaussianBlur in="SourceAlpha" stdDeviation="{blur}" result="dropBlur"/>
<feOffset in="dropBlur" dx="{d}" dy="{dy}" result="dropOff"/>
<feFlood flood-color="{shadow}" flood-opacity="{}" result="dropColor"/>
<feComposite in="dropColor" in2="dropOff" operator="in" result="drop"/>
<feOffset in="SourceAlpha" dx="{d}" dy="{d}" result="oL"/>
<feGaussianBlur in="oL" stdDeviation="{d}" result="bL"/>
<feComposite in="SourceAlpha" in2="bL" operator="out" result="rimL"/>
<feFlood flood-color="#ffffff" flood-opacity="{}" result="fL"/>
<feComposite in="fL" in2="rimL" operator="in" result="light"/>
<feOffset in="SourceAlpha" dx="-{d}" dy="-{d}" result="oD"/>
<feGaussianBlur in="oD" stdDeviation="{d}" result="bD"/>
<feComposite in="SourceAlpha" in2="bD" operator="out" result="rimD"/>
<feFlood flood-color="{shadow}" flood-opacity="{}" result="fD"/>
<feComposite in="fD" in2="rimD" operator="in" result="dark"/>
<feOffset in="SourceAlpha" dx="2" dy="2" result="oS"/>
<feGaussianBlur in="oS" stdDeviation="1.2" result="bS"/>
<feComposite in="SourceAlpha" in2="bS" operator="out" result="rimS"/>
<feFlood flood-color="#ffffff" flood-opacity="0.9" result="fS"/>
<feComposite in="fS" in2="rimS" operator="in" result="spec"/>
<feMerge result="clay"><feMergeNode in="drop"/><feMergeNode in="SourceGraphic"/><feMergeNode in="light"/><feMergeNode in="dark"/><feMergeNode in="spec"/></feMerge>
"##,
        c.id,
        c.drop_op,
        c.light_op,
        c.dark_op,
        shadow = c.shadow,
    )
    .unwrap();
    if c.grain > 0.0 {
        write!(
            f,
            r#"<feTurbulence type="fractalNoise" baseFrequency="0.8" numOctaves="3" seed="7" stitchTiles="stitch" result="noise"/>
<feColorMatrix in="noise" type="saturate" values="0" result="mono"/>
<feComponentTransfer in="mono" result="grainA"><feFuncR type="linear" slope="4" intercept="-1.5"/><feFuncG type="linear" slope="4" intercept="-1.5"/><feFuncB type="linear" slope="4" intercept="-1.5"/><feFuncA type="linear" slope="{:.3}"/></feComponentTransfer>
<feComposite in="grainA" in2="SourceAlpha" operator="in" result="grainIn"/>
<feBlend in="grainIn" in2="clay" mode="overlay"/>
"#,
            c.grain
        )
        .unwrap();
    }
    f.push_str("</filter>");
    f
}

fn grad(id: &str, a: &str, b: &str) -> String {
    format!(
        r#"<linearGradient id="{id}" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="{a}"/><stop offset="1" stop-color="{b}"/></linearGradient>"#
    )
}

// ---- the mark, in its 400-unit space ----------------------------------------

struct Parts {
    defs: String,
    tile: String,
    glyph: String,
}

/// `p` prefixes the ids so two marks can share one document.
fn parts(m: &Mode, g: &Glyph, grain: f64, p: &str) -> Parts {
    let tile_top = if m.dark {
        mix(m.tile, "#ffffff", 0.18)
    } else {
        tint(m.tile)
    };
    let tile_clay = if m.dark {
        Clay {
            id: format!("{p}fTile"),
            d: 14,
            shadow: "#0d0a1f",
            light_op: 0.35,
            dark_op: 0.6,
            drop_op: 0.45,
            grain,
        }
    } else {
        Clay {
            id: format!("{p}fTile"),
            d: 12,
            shadow: &shade(m.tile),
            light_op: 0.85,
            dark_op: 0.45,
            drop_op: 0.28,
            grain,
        }
    };
    let sign_clay = Clay {
        id: format!("{p}fSign"),
        d: 7,
        shadow: &shade(m.mark),
        light_op: 0.85,
        dark_op: 0.45,
        drop_op: 0.3,
        grain: grain * 0.8,
    };
    let defs = grad(&format!("{p}gTile"), &tile_top, m.tile)
        + &grad(&format!("{p}gSign"), &tint(m.mark), m.mark)
        + &clay(&tile_clay)
        + &clay(&sign_clay);
    let tile = format!(
        r#"<path d="{}" fill="url(#{p}gTile)" filter="url(#{p}fTile)"/>"#,
        *TILE_PATH
    );
    let transform = if g.placed {
        String::new()
    } else {
        let s = INK / g.w.max(g.h);
        let (cx, cy) = (g.x + g.w / 2.0, g.y + g.h / 2.0);
        format!(
            r#" transform="translate(200 200) scale({s:.4}) translate({:.2} {:.2})""#,
            -cx, -cy
        )
    };
    let body = g.body.replace("currentColor", &format!("url(#{p}gSign)"));
    let glyph =
        format!(r#"<g filter="url(#{p}fSign)"><g{transform} fill="url(#{p}gSign)">{body}</g></g>"#);
    Parts { defs, tile, glyph }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn svg(g: &Glyph, o: &Options) -> String {
    let view = match o.crop {
        Crop::Full => FULL_BOX,
        Crop::Tile => TILE_BOX,
    };
    let label = escape(&o.label);
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{view}\" role=\"img\" aria-label=\"{label}\">\n<title>{label}</title>\n"
    );
    if let Some(n) = &g.notice {
        out.push_str(n);
        out.push('\n');
    }
    match o.theme {
        Theme::Auto => {
            let l = parts(&LIGHT, g, o.grain, "l");
            let d = parts(&DARK, g, o.grain, "d");
            out.push_str("<style>.dark{display:none}@media (prefers-color-scheme:dark){.light{display:none}.dark{display:inline}}</style>\n");
            out.push_str(&format!(
                "<defs>{}{}</defs>\n<g class=\"light\">{}{}</g>\n<g class=\"dark\">{}{}</g>\n",
                l.defs, d.defs, l.tile, l.glyph, d.tile, d.glyph
            ));
        }
        theme => {
            let m = if theme == Theme::Dark { &DARK } else { &LIGHT };
            let p = parts(m, g, o.grain, "");
            out.push_str(&format!(
                "<defs>{}</defs>\n<g>{}{}</g>\n",
                p.defs, p.tile, p.glyph
            ));
        }
    }
    out.push_str("</svg>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_matches_the_shipped_marks() {
        // The gradient stops in expenses' web/public/logo.svg.
        assert_eq!(tint("#b4a3f2"), "#ccc0f6");
        assert_eq!(tint("#fdf3e4"), "#fef7ed");
        assert_eq!(shade("#b4a3f2"), "#5f5287");
    }

    #[test]
    fn tile_spans_the_expected_box() {
        assert!(TILE_PATH.starts_with("M 360.0 200.0 L "));
        assert!(TILE_PATH.contains(" 200.0 360.0 "));
    }

    #[test]
    fn grain_is_optional() {
        let g = Glyph {
            body: "<path d=\"M0 0h10v10H0z\"/>".into(),
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
            placed: false,
            notice: None,
        };
        let with = svg(
            &g,
            &Options {
                theme: Theme::Light,
                crop: Crop::Full,
                grain: 1.0,
                label: "x".into(),
            },
        );
        let without = svg(
            &g,
            &Options {
                theme: Theme::Light,
                crop: Crop::Full,
                grain: 0.0,
                label: "x".into(),
            },
        );
        assert!(with.contains("<feTurbulence"));
        assert!(!without.contains("<feTurbulence"));
        // Fitted: 10 units become 184.
        assert!(with.contains("scale(18.4000)"));
    }

    #[test]
    fn label_is_escaped() {
        let g = Glyph {
            body: String::new(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            placed: true,
            notice: None,
        };
        let s = svg(
            &g,
            &Options {
                theme: Theme::Dark,
                crop: Crop::Tile,
                grain: 0.0,
                label: "a<b".into(),
            },
        );
        assert!(s.contains("aria-label=\"a&lt;b\""));
        assert!(s.contains("viewBox=\"32 32 336 336\""));
    }
}
