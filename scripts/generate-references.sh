#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
FIXTURES_DIR="$REPO_DIR/tests/fixtures"
REF_DIR="$FIXTURES_DIR/references"

# Find Chrome
CHROME=$(command -v google-chrome-stable || command -v google-chrome || command -v chromium || command -v chromium-browser || echo "")
if [ -z "$CHROME" ]; then
    echo "Warning: Chrome/Chromium not found — skipping reference generation"
    exit 0
fi

echo "Using: $CHROME"
count=0

for layer in features combined edge-cases; do
    mkdir -p "$REF_DIR/$layer"
    for html_file in "$FIXTURES_DIR/$layer"/*.html; do
        [ -f "$html_file" ] || continue
        name=$(basename "$html_file" .html)
        output="$REF_DIR/$layer/$name.png"
        echo "  $layer/$name..."
        "$CHROME" --headless=new --disable-gpu --no-sandbox --disable-software-rasterizer \
            --window-size=1240,1754 \
            --screenshot="$output" \
            "file://$html_file" 2>/dev/null || \
        "$CHROME" --headless --disable-gpu --no-sandbox \
            --window-size=1240,1754 \
            --screenshot="$output" \
            "file://$html_file" 2>/dev/null || \
        echo "    WARN: failed to screenshot $layer/$name"
        [ -f "$output" ] && count=$((count + 1))
    done
done

echo "Done. $count reference PNGs saved to $REF_DIR"
