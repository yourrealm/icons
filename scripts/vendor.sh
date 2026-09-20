#!/usr/bin/env bash
# Vendor the glyph sources into server/assets so the binary embeds them.
# Pinned versions; bump here and rerun (`pnpm vendor`). Needs npm and curl.
set -euo pipefail
cd "$(dirname "$0")/.."
FA=7.3.1
PH=2.1.1
NUNITO_REF=main
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

(cd "$tmp" && npm pack "@fortawesome/fontawesome-free@$FA" "@phosphor-icons/core@$PH" --silent >/dev/null)
for t in "$tmp"/*.tgz; do d="${t%.tgz}"; mkdir -p "$d"; tar xzf "$t" -C "$d"; done

rm -rf server/assets/fa server/assets/ph server/assets/font
mkdir -p server/assets/fa server/assets/ph server/assets/font

# Font Awesome Free, the solid style. Icons are CC BY 4.0: the comment in each
# file is the attribution and the renderer keeps it in every mark it emits.
cp "$tmp"/fortawesome-fontawesome-free-*/package/svgs/solid/*.svg server/assets/fa/
cp "$tmp"/fortawesome-fontawesome-free-*/package/LICENSE.txt server/assets/fa/LICENSE.txt

# Phosphor, the fill weight (MIT). Files are named `name-fill.svg`; drop the suffix.
for f in "$tmp"/phosphor-icons-core-*/package/assets/fill/*-fill.svg; do
  b=$(basename "$f" -fill.svg); cp "$f" "server/assets/ph/$b.svg"
done
cp "$tmp"/phosphor-icons-core-*/package/LICENSE server/assets/ph/LICENSE

# Nunito, the variable font from Google Fonts (OFL). The renderer sets wght=900.
curl -sSL "https://raw.githubusercontent.com/google/fonts/$NUNITO_REF/ofl/nunito/Nunito%5Bwght%5D.ttf" -o "server/assets/font/Nunito[wght].ttf"
curl -sSL "https://raw.githubusercontent.com/google/fonts/$NUNITO_REF/ofl/nunito/OFL.txt" -o server/assets/font/OFL.txt

echo "fa: $(ls server/assets/fa/*.svg | wc -l | tr -d ' ') icons, ph: $(ls server/assets/ph/*.svg | wc -l | tr -d ' ') icons, font: $(stat -f %z "server/assets/font/Nunito[wght].ttf") bytes"
