#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$repo_root"

only_allow="$(sed -n '/^accepted = \[/,/^\]/s/^[[:space:]]*"\(.*\)",\{0,1\}$/\1/p' about.toml | paste -sd ';' -)"
if [ -z "$only_allow" ]; then
    echo "about.toml contains no accepted licenses" >&2
    exit 1
fi

install -d frontend/build-assets/fonts
cp frontend/node_modules/@expo-google-fonts/gelasio/400Regular/Gelasio_400Regular.ttf \
    frontend/build-assets/fonts/Gelasio_400Regular.ttf
cp frontend/node_modules/@expo-google-fonts/gelasio/400Regular_Italic/Gelasio_400Regular_Italic.ttf \
    frontend/build-assets/fonts/Gelasio_400Regular_Italic.ttf

cd frontend
npx license-checker-rseidelsohn \
    --production \
    --excludePackages koshelf-frontend \
    --onlyAllow "$only_allow" \
    --customPath license-checker-format.json \
    --json \
    --out build-assets/npm-licenses.json
