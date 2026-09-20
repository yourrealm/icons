# realm-icons

Realm's clay mark with any glyph on it, served by URL, and a web UI to pick one. For the Realm apps
that want a mark of their own without a designer.

```
GET /char/÷.svg              a character, set in Nunito Black
GET /fa/piggy-bank.svg       Font Awesome Free, the solid style
GET /ph/wallet.svg           Phosphor, the fill weight
GET /shape/divide.svg        the built-in shapes: divide (expenses), ring (loans)

GET /fa/piggy-bank.png?size=512      16 to 1024 px, default 512
GET /fa/piggy-bank.ico               16, 32 and 48 px frames, tile crop, no grain

?theme=light|dark|auto      default light; auto is both marks in one SVG, picked by prefers-color-scheme
?crop=full|tile             default full (the 400-unit mark with its shadow); tile fills the frame, for icons
?grain=0..1                 default 1, the heavy grain; PNGs at or below 64 px and ICO drop it
?label=text                 the SVG's title and aria-label, default the glyph's name

GET /api/sets               the icon names in each set, for the UI
GET /healthz
```

Responses carry `Cache-Control: public, max-age=604800`, an ETag and
`Access-Control-Allow-Origin: *`, so hotlinking from a `realm.tsx` is fine. Errors are
`{ "error": "..." }` with 400 or 404.

The web UI at `/` picks a source and a glyph, previews both themes, and gives the five files an app
ships (`logo.svg`, `logo-dark.svg`, `favicon.svg`, `favicon.ico`, `icon-512.png`) as downloads and
as URLs to copy.

## Run

```
pnpm vendor        # once: fetch the font and icon sets into server/assets
pnpm dev:server    # builds web/, then the server on :3000 (ICONS_PORT moves it)
pnpm dev:web       # vite on :5173, proxying the image routes to the server
pnpm typecheck     # tsc, vite build, clippy
pnpm test          # cargo test
pnpm fmt           # deno fmt, cargo fmt
pnpm build:image   # docker build -t realm-icons:latest .
```

The binary needs nothing at run time: the web UI, the font and both icon sets are embedded.
Rendering is pure Rust (resvg), so PNG and ICO come out the same on every machine.

## Deploy

CI pushes `ghcr.io/yourrealm/icons:latest` on every push to main.

```yaml
services:
  icons:
    image: ghcr.io/yourrealm/icons:latest
    restart: unless-stopped
    ports: ["3000:3000"]
```

The image has its own healthcheck (`realm-icons healthcheck`, no shell needed).

## Licenses

The code is MIT. The glyph sources are vendored under `server/assets` with their licenses and
embedded in the binary:

- Nunito (OFL 1.1), the variable font from Google Fonts, set to weight 900.
- Font Awesome Free 7 icons (CC BY 4.0). The attribution comment in each icon is carried into every
  SVG made from it; keep it when you ship one.
- Phosphor 2 icons (MIT).
