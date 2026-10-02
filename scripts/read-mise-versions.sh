#!/usr/bin/env bash
set -euo pipefail

read_pinned_version() {
  awk -F '"' -v key="$1" '
    /^\[tools\]$/ { in_tools = 1; next }
    /^\[/ { in_tools = 0 }
    in_tools && $1 == key " = " {
      count++
      version = $2
    }
    END {
      if (count != 1 || version !~ /^[0-9]+\.[0-9]+\.[0-9]+$/) exit 1
      print version
    }
  ' mise.toml
}

printf 'node=%s\n' "$(read_pinned_version node)"
printf 'rust=%s\n' "$(read_pinned_version rust)"
