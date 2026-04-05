#!/usr/bin/env bash
set -euo pipefail

PROJECT_NAME="dobby-cli"
BINARY_NAME="dobby"
REPO_URL="${DOBBY_REPO:-https://github.com/shibinsp/Dobby.git}"
DEFAULT_BRANCH="main"
FORCE_INSTALL=1
DRY_RUN=0
REF_TYPE="branch"
REF_VALUE="${DOBBY_REF:-${DOBBY_BRANCH:-${DOBBY_TAG:-${DOBBY_VERSION:-}}}}"

usage() {
    cat <<'EOF'
Dobby CLI installer

Usage:
  curl -fsSL https://raw.githubusercontent.com/shibinsp/Dobby/main/scripts/install.sh | bash

Options:
  -b, --branch <name>   Install from a specific branch (default: main)
  -t, --tag <tag>       Install from a git tag
      --rev <sha>       Install from a specific commit SHA
      --no-force        Do not pass --force to cargo install
      --dry-run         Print the cargo command without executing it
  -h, --help            Show this message

Environment overrides:
  DOBBY_REPO            Alternate git repository URL
  DOBBY_BRANCH          Default branch name
  DOBBY_TAG             Default tag
  DOBBY_VERSION         Alias for DOBBY_TAG
  DOBBY_REF             Shortcut for DOBBY_TAG (takes precedence)
EOF
}

log() {
    printf '[dobby-install] %s\n' "$1"
}

warn() {
    printf '[dobby-install] warning: %s\n' "$1" >&2
}

fatal() {
    printf '[dobby-install] error: %s\n' "$1" >&2
    exit 1
}

require_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        fatal "Missing required command: $1"
    fi
}

BRANCH_OVERRIDE="${DOBBY_BRANCH:-}"
TAG_OVERRIDE="${DOBBY_TAG:-${DOBBY_VERSION:-}}"
REV_OVERRIDE="${DOBBY_REV:-}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        -b|--branch)
            [[ $# -lt 2 ]] && fatal "--branch requires a value"
            REF_TYPE="branch"
            REF_VALUE="$2"
            shift 2
            ;;
        -t|--tag)
            [[ $# -lt 2 ]] && fatal "--tag requires a value"
            REF_TYPE="tag"
            REF_VALUE="$2"
            shift 2
            ;;
        --rev)
            [[ $# -lt 2 ]] && fatal "--rev requires a value"
            REF_TYPE="rev"
            REF_VALUE="$2"
            shift 2
            ;;
        --no-force)
            FORCE_INSTALL=0
            shift
            ;;
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            fatal "Unknown option: $1"
            ;;
    esac
done

if [[ -n "$REV_OVERRIDE" ]]; then
    REF_TYPE="rev"
    REF_VALUE="$REV_OVERRIDE"
elif [[ -n "$TAG_OVERRIDE" ]]; then
    REF_TYPE="tag"
    REF_VALUE="$TAG_OVERRIDE"
elif [[ -n "$BRANCH_OVERRIDE" ]]; then
    REF_TYPE="branch"
    REF_VALUE="$BRANCH_OVERRIDE"
fi

if [[ -z "$REF_VALUE" ]]; then
    REF_VALUE="$DEFAULT_BRANCH"
    REF_TYPE="branch"
fi

if [[ "$REF_TYPE" != "branch" && "$REF_TYPE" != "tag" && "$REF_TYPE" != "rev" ]]; then
    fatal "Invalid reference type: $REF_TYPE"
fi

require_cmd cargo
require_cmd git
if ! command -v protoc >/dev/null 2>&1; then
    warn "Protocol Buffers compiler (protoc) not found. Forge delegation build may fail."
fi

CARGO_ARGS=(install --locked --git "$REPO_URL")
case "$REF_TYPE" in
    branch)
        CARGO_ARGS+=(--branch "$REF_VALUE")
        ;;
    tag)
        CARGO_ARGS+=(--tag "$REF_VALUE")
        ;;
    rev)
        CARGO_ARGS+=(--rev "$REF_VALUE")
        ;;
esac

if (( FORCE_INSTALL )); then
    CARGO_ARGS+=(--force)
fi

CARGO_ARGS+=("$PROJECT_NAME")

log "Installing $BINARY_NAME from $REPO_URL ($REF_TYPE: $REF_VALUE)"
if (( DRY_RUN )); then
    log "Dry run enabled. Command: cargo ${CARGO_ARGS[*]}"
    exit 0
fi

cargo "${CARGO_ARGS[@]}"

log "Installation complete. Ensure \"$HOME/.cargo/bin\" is on your PATH to use '$BINARY_NAME'."
