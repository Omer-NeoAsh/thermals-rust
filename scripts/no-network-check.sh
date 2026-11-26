#!/usr/bin/env bash

set -u

case "$(uname -s)" in
Linux)
  trace=$(mktemp)
  strace -f -qq -e trace=socket,socketpair -o "$trace" "$@"
  status=$?
  if grep -E 'socket(pair)?\(AF_' "$trace" | grep -qv 'AF_UNIX'; then
    cat "$trace" >&2
    echo "no-network-check: the command created a network socket." >&2
    exit 3
  fi
  exit "$status"
  ;;
Darwin)
  sandbox-exec -p '(version 1) (allow default) (deny network* (with send-signal SIGKILL))' "$@"
  status=$?
  if [ "$status" -eq 137 ]; then
    echo "no-network-check: the sandbox killed the command for using the network." >&2
    exit 3
  fi
  exit "$status"
  ;;
*)
  echo "no-network-check: unsupported OS $(uname -s)" >&2
  exit 2
  ;;
esac