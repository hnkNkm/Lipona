{
  description = "Lipona - A minimal programming language based on Toki Pona grammar";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain

            # Build tools
            cargo-watch
            cargo-edit
            cargo-expand

            # For parsing (optional, useful for language development)
            # nom or pest are pure Rust, no extra deps needed

            # Useful for debugging
            lldb
          ];

          shellHook = ''
            echo "🌱 Lipona development environment loaded"
            echo "Rust: $(rustc --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo build    - Build the project"
            echo "  cargo run      - Run the interpreter"
            echo "  cargo test     - Run tests"
            echo "  cargo watch -x run  - Auto-rebuild on changes"
          '';

          RUST_BACKTRACE = 1;
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "lipona";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      }
    );
}
