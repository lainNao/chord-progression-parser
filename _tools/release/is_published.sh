#!/usr/bin/env bash
# Query the public registry, independently of local workspace metadata.
# Print true/false only for verified metadata or a genuine HTTP 404.
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "Usage: is_published.sh <npm|crates-io> <name> <version>" >&2
  exit 1
fi

registry="$1"
package_name="$2"
package_version="$3"
case "$registry" in
  npm)
    # Encode the slash in scoped npm package names.
    url="https://registry.npmjs.org/${package_name//\//%2F}/${package_version}"
    # These variables are expanded by jq rather than the shell.
    # shellcheck disable=SC2016
    selector='.name == $name and .version == $version'
    ;;
  crates-io)
    url="https://crates.io/api/v1/crates/${package_name}/${package_version}"
    # These variables are expanded by jq rather than the shell.
    # shellcheck disable=SC2016
    selector='.version.crate == $name and .version.num == $version'
    ;;
  *)
    echo "Unsupported registry: $registry" >&2
    exit 1
    ;;
esac

response_file="$(mktemp)"
trap 'rm -- "$response_file"' EXIT
http_status="$(curl --silent --show-error --location \
  --connect-timeout 15 --max-time 60 --retry 3 \
  --user-agent 'chord-progression-parser release (https://github.com/lainNao/chord-progression-parser)' \
  --output "$response_file" --write-out '%{http_code}' "$url")"

case "$http_status" in
  200)
    if ! jq --exit-status --arg name "$package_name" --arg version "$package_version" \
      "$selector" "$response_file" >/dev/null; then
      echo "Registry returned invalid metadata for ${package_name}@${package_version}" >&2
      exit 1
    fi
    echo true
    ;;
  404)
    echo false
    ;;
  *)
    echo "Registry request failed with HTTP ${http_status}: ${package_name}@${package_version}" >&2
    cat "$response_file" >&2
    exit 1
    ;;
esac
