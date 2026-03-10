#!/bin/sh
set -eu

REPO="brynary/arc"

# Colors (only when stderr is a terminal)
if [ -t 2 ]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  DIM='\033[2m'
  BOLD='\033[1m'
  BOLD_CYAN='\033[1;36m'
  RESET='\033[0m'
else
  RED=''
  GREEN=''
  DIM=''
  BOLD=''
  BOLD_CYAN=''
  RESET=''
fi

info()    { printf "  %s\n" "$1" >&2; }
step()    { printf "  ${BOLD}%s${RESET}\n" "$1" >&2; }
dim()     { printf "  ${DIM}%s${RESET}\n" "$1" >&2; }
success() { printf "  ${GREEN}✔${RESET} %s\n" "$1" >&2; }
error()   { printf "  ${RED}✗ %s${RESET}\n" "$1" >&2; exit 1; }

# --- Header ---
printf "\n  ⚒️  ${BOLD}Fabro Install${RESET}\n\n" >&2

# --- Require gh CLI ---
if ! command -v gh >/dev/null 2>&1; then
  error "gh CLI is required but not installed. Install it from ${BOLD_CYAN}https://cli.github.com${RESET}"
fi

# --- Detect platform ---
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin)
    # Detect Rosetta translation
    if [ "$ARCH" = "x86_64" ]; then
      if sysctl -n sysctl.proc_translated 2>/dev/null | grep -q 1; then
        ARCH="arm64"
      fi
    fi
    case "$ARCH" in
      arm64) TARGET="aarch64-apple-darwin" ;;
      *)     error "Unsupported macOS architecture: $ARCH. Supported: Apple Silicon (arm64)" ;;
    esac
    ;;
  Linux)
    case "$ARCH" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      *)      error "Unsupported Linux architecture: $ARCH. Supported: x86_64" ;;
    esac
    ;;
  *)
    error "Unsupported OS: $OS. Supported platforms: macOS (Apple Silicon), Linux (x86_64)"
    ;;
esac

ASSET="arc-${TARGET}.tar.gz"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

dim "Downloading arc for ${TARGET}..."
gh release download --repo "$REPO" --pattern "$ASSET" --dir "$TMPDIR" --clobber

dim "Extracting..."
tar xzf "${TMPDIR}/${ASSET}" -C "$TMPDIR"

# --- Install binary ---
INSTALL_DIR="${ARC_INSTALL_DIR:-/usr/local/bin}"

if [ -w "$INSTALL_DIR" ]; then
  mv "${TMPDIR}/arc-${TARGET}/arc" "${INSTALL_DIR}/arc"
else
  dim "Installing to ${INSTALL_DIR} (requires sudo)..."
  sudo mv "${TMPDIR}/arc-${TARGET}/arc" "${INSTALL_DIR}/arc"
fi

chmod +x "${INSTALL_DIR}/arc"

# --- Verify ---
VERSION="$("${INSTALL_DIR}/arc" --version 2>/dev/null || true)"
if [ -z "$VERSION" ]; then
  error "Installation failed: could not run arc --version"
fi

success "Installed ${VERSION} to ${BOLD_CYAN}${INSTALL_DIR}/arc${RESET}"
echo "" >&2

# --- Prompt to run setup wizard ---
if [ -t 0 ] && [ -t 2 ]; then
  printf "  ${BOLD}Run ${BOLD_CYAN}arc install${RESET}${BOLD} now to complete setup? [Y/n]${RESET} " >&2
  read -r answer </dev/tty
  case "$answer" in
    [nN]*) dim "Skipping. Run ${BOLD_CYAN}arc install${RESET}${DIM} whenever you're ready." ;;
    *)     echo "" >&2; exec "${INSTALL_DIR}/arc" install ;;
  esac
else
  info "Run ${BOLD_CYAN}arc install${RESET} to complete setup."
fi
