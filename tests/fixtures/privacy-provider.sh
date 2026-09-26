#!/bin/sh
# Synthetic direct-child privacy probe. No external commands or source writes.
set -eu
[ "$#" -eq 1 ] || exit 81
[ "$1" = "FLOW_ARGV_CANARY_7429" ] || exit 82
[ "${HOME+x}" != x ] || exit 83
[ "$FLOW_PRIVATE" = "FLOW_ENV_CANARY_1938 FLOW_SECRET_CANARY_5813 FLOW_PRIVATE_CANARY_2604" ] || exit 84
IFS= read -r request || exit 85
printf "%s" "$FLOW_PROVIDER_STDOUT"
printf "%s %s" "$1" "$FLOW_PRIVATE" >&2
