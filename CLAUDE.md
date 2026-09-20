# CLAUDE.md

Guidance for Claude Code working in this repository.

## What this is

**realm-icons** serves Realm's clay mark with any glyph on it, by URL, with a web UI for picking
one. It is a plain Docker image, not a Realm app (yet): `icons.yourrealm.eu` runs it from docker
compose on the Realm cloud box. The mark is the one Realm (`../holden/scripts/logo/gen.ts`),
expenses (`../pengar`) and loans (`../loans`) ship; those repos still generate their own files and
are the reference for how a mark should look.

Built 2026-09-20. The README has the URL scheme and commands; this file has the decisions.

## Decisions

1. **URL = what, extension = format, query = how.** `/{set}/{name}.{svg|png|ico}` with `theme`,
   `crop`, `grain`, `label`, `size` in the query. Same URL, same image. One character per `/char/`
   URL, percent-encoded.
2. **Sets:** `char` (Nunito Black, the font Realm's R is), `fa` (Font Awesome Free solid), `ph`
   (Phosphor fill), `shape` (divide, ring). Chosen 2026-09-20 over Lucide (thin strokes turn to mush
   under the clay), Material Symbols and Heroicons (fine interior detail). Filled, rounded glyphs
   are what the clay wants.
3. **Fitting:** icons and characters are scaled so their ink box's longer side is 184 units on the
   320-unit tile, centred. Shapes are placed as drawn, so they match what expenses and loans ship.
   Ink boxes come from usvg for icons and from ttf-parser for characters.
4. **Grain is heavy by default** (`grain=1`): fractal noise at frequency 0.8, three octaves,
   contrast stretched 4x around mid-grey, overlaid on the clay and clipped to the shape; the glyph
   gets 0.8 of the tile's amount. The stretch matters: raw fractal noise sits near 0.5 and overlay
   with mid-grey is a no-op, so browsers showed almost nothing until it was added (tuned in Chromium
   2026-09-20; resvg alone looks heavier than browsers do). PNGs at 64 px and below and every ICO
   drop it, since it is only noise there. `grain=0` gives the original marks.
5. **Pure Rust rendering.** resvg for PNG, the `ico` crate for favicons, ttf-parser for outlines. No
   librsvg, no ImageMagick, no fonts on the system: the font and both icon sets are embedded with
   rust-embed.
6. **Cache a week, not forever.** `max-age=604800` plus an ETag. The URL does not carry a recipe
   version, so a change to the clay or grain must reach hotlinks without renaming them.
7. **Attribution travels.** Font Awesome's icons are CC BY 4.0. Their comment is lifted out of the
   icon file and written once at the top of every SVG made from it. Phosphor is MIT, Nunito is OFL;
   all three license files sit next to the assets.
8. **The UI downloads, `realm.tsx` hotlinks.** Apps copy the five files into their `web/public`; a
   favicon must not depend on this service being up. Only a deployment description should point at a
   URL here.

## Layout

```
server/            Rust crate: axum, the mark builder, glyph sources, resvg
server/assets/     vendored font and icon sets, written by scripts/vendor.sh (committed)
web/               React + Vite UI, embedded into the binary from web/dist
scripts/vendor.sh  pins the set versions; rerun to bump them
Dockerfile         node → rust → distroless, plain docker build
.github/workflows  check, then push the image to GHCR on main
```

`server/src/mark.rs` is the port of expenses' generator: same palette, tile, clay filter. `glyph.rs`
turns a source into a `Glyph` (body, ink box). `render.rs` rasterises. `api.rs` parses the URL and
query and sets headers.

`cargo` needs `web/dist` to exist (rust-embed reads it at compile time; debug builds read it from
disk at run time). The root scripts build web first.

## Conventions

As in the Realm repo: short comments, no compat shims, don't export what nothing imports, plain
language in docs. Deno formats the web code and the markdown; cargo fmt the Rust. Don't commit or
push unless asked.
