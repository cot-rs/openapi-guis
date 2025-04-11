#!/bin/bash

set -e -o pipefail

if [ -z "$1" ]; then
    echo "USAGE: $0 <version>"
    echo "Example: $0 v5.20.8"
    exit 1
fi

VERSION="$1"

curl -o res/LICENSE https://raw.githubusercontent.com/swagger-api/swagger-ui/refs/tags/$VERSION/LICENSE
curl -o res/swagger-ui.css https://raw.githubusercontent.com/swagger-api/swagger-ui/refs/tags/$VERSION/dist/swagger-ui.css
curl -o res/index.css https://raw.githubusercontent.com/swagger-api/swagger-ui/refs/tags/$VERSION/dist/index.css
curl -o res/swagger-ui-bundle.js https://raw.githubusercontent.com/swagger-api/swagger-ui/refs/tags/$VERSION/dist/swagger-ui-bundle.js
curl -o res/swagger-ui-standalone-preset.js https://raw.githubusercontent.com/swagger-api/swagger-ui/refs/tags/$VERSION/dist/swagger-ui-standalone-preset.js
curl -o res/favicon-16x16.png https://raw.githubusercontent.com/swagger-api/swagger-ui/refs/tags/$VERSION/dist/favicon-16x16.png
curl -o res/favicon-32x32.png https://raw.githubusercontent.com/swagger-api/swagger-ui/refs/tags/$VERSION/dist/favicon-32x32.png

sed -i "s/<!-- version -->.*$/<!-- version -->The version of Swagger UI included in this crate is $VERSION./" README.md
