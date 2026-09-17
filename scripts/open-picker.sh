#!/usr/bin/env bash
set -o errexit
set -o nounset
set -o pipefail

# Summon the picker popup declared as the "picker" pane entrypoint in
# herdr-plugin.toml. Placement (popup + size) is set in the manifest.
HERDR="${HERDR_BIN_PATH:-herdr}"
exec "$HERDR" plugin pane open \
  --plugin "${HERDR_PLUGIN_ID:-herdr-repo-picker}" \
  --entrypoint picker
