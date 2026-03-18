{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pre-commit
            bun
            jq
            playwright-driver.browsers
            pkg-config
            openssl
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          PLAYWRIGHT_BROWSERS_PATH = "${pkgs.playwright-driver.browsers}";
          PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = "true";

          shellHook = ''
            # Create local dependency directories
            mkdir -p .dependencies/bun
            mkdir -p .dependencies/rust

            # Configure Bun to use local directory
            export BUN_INSTALL="$PWD/.dependencies/bun"
            export PATH="$PWD/.dependencies/bun/bin:$PATH"

            # Configure Cargo to use local directory
            export CARGO_HOME="$PWD/.dependencies/rust/cargo"
            export RUSTUP_HOME="$PWD/.dependencies/rust/rustup"
            export PATH="$PWD/.dependencies/rust/cargo/bin:$PATH"

            CARGO_AUDIT_VERSION="0.22.1"
            CARGO_DENY_VERSION="0.19.0"
            CARGO_NEXTEST_VERSION="0.9.130"
            CARGO_LEPTOS_VERSION="0.3.5"

            mkdir -p $BUN_INSTALL/bin
            mkdir -p $CARGO_HOME
            mkdir -p $RUSTUP_HOME


            # Check cargo-nextest version
            if ! command -v cargo-nextest >/dev/null 2>&1 || [ "$(cargo-nextest --version 2>/dev/null | awk '{print $2}')" != "$CARGO_NEXTEST_VERSION" ]; then
              echo "Installing cargo-nextest $CARGO_NEXTEST_VERSION"
              cargo install cargo-nextest --version "$CARGO_NEXTEST_VERSION" --locked
            fi

            # Check cargo-audit version
            if ! command -v cargo-audit >/dev/null 2>&1 || [ "$(cargo-audit --version 2>/dev/null | awk '{print $2}')" != "$CARGO_AUDIT_VERSION" ]; then
              echo "Installing cargo-audit $CARGO_AUDIT_VERSION"
              cargo install cargo-audit --version "$CARGO_AUDIT_VERSION"
            fi

            # Check cargo-deny version
            if ! command -v cargo-deny >/dev/null 2>&1 || [ "$(cargo-deny --version 2>/dev/null | awk '{print $2}')" != "$CARGO_DENY_VERSION" ]; then
              echo "Installing cargo-deny $CARGO_DENY_VERSION"
              cargo install cargo-deny --version "$CARGO_DENY_VERSION" --locked
            fi

            # Check cargo-leptos version
            if ! command -v cargo-leptos >/dev/null 2>&1 || [ "$(cargo-leptos --version 2>/dev/null | awk '{print $2}')" != "$CARGO_LEPTOS_VERSION" ]; then
              echo "Installing cargo-leptos $CARGO_LEPTOS_VERSION"
              cargo install cargo-leptos --version "$CARGO_LEPTOS_VERSION"
            fi

            # Use project-local advisory database
            alias cargo-audit='cargo audit --db "$PWD/.cargo-advisory-db"'
          '';
        };
      }
    );
}
