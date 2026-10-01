#!/usr/bin/env bash
# Report non-ASCII source lines; an empty result is not a command failure.
# Usage: ./_tools/find_files_include_multibyte_characters.sh
set -euo pipefail

# LC_ALL takes precedence over inherited locale settings. Recursive grep also
# keeps filenames containing spaces intact instead of splitting find output.
status=0
LC_ALL=C grep -rHnv --binary-files=without-match '^[[:cntrl:][:print:]]*$' src || status=$?
case "$status" in
  0|1) exit 0 ;;
  *) exit "$status" ;;
esac
