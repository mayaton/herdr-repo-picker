#!/usr/bin/env bash
set -o errexit
set -o nounset
set -o pipefail

HERDR="${HERDR_BIN_PATH:-herdr}"
exec "$HERDR" plugin pane open \
  --plugin herdr-repo-picker \
  --entrypoint picker \
  --placement overlay \
  --focus
