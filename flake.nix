{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        rustToolchainFor =
          p:
          p.rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" ];
          };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainFor;

        unfilteredRoot = ./.;
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            # additional asset files
            (lib.fileset.fileFilter (
              file:
              lib.any file.hasExt [
                "html"
                "css"
              ]
            ) unfilteredRoot)
            # markdown files in the source tree
            (lib.fileset.fileFilter (
              file: lib.any file.hasExt ["md"]
            ) ./src)
            # static assets
            ./static
            # tailwind config
            ./tailwind.config.js
          ];
        };

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;
          strictDeps = true;
          # We must force the target, otherwise cargo will attempt to use your native target
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";

          buildInputs =
            [
              pkgs.binaryen
              pkgs.tailwindcss
              # Add additional build inputs here
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              pkgs.libiconv
            ];
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly (
          commonArgs
          // {
            # You cannot run cargo test on a wasm build
            doCheck = false;
          }
        );

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        # This derivation is a directory you can put on a webserver.
        diff-rs = craneLib.buildTrunkPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            wasm-bindgen-cli = pkgs.buildWasmBindgenCli rec {
              src = pkgs.fetchCrate {
                pname = "wasm-bindgen-cli";
                # The version of wasm-bindgen-cli here must match the one from Cargo.lock.
                version = "0.2.130";
                hash = "sha256-9Og1O/6/VC3sc5hBjoIWBErmLW2UHGMsTQO+6+m/dng=";
              };

              cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
                inherit src;
                inherit (src) pname version;
                hash = "sha256-j81sUf1m5yXU4FJfGj7GV+hdqVCV8mRmNb0QIsMCjqw=";
              };
            };
            trunkExtraBuildArgs = "--verbose --verbose --offline true";
            # Trunk.toml pins an exact tailwindcss version, use the one from nixpkgs instead.
            TRUNK_TOOLS_TAILWINDCSS = pkgs.tailwindcss.version;
            nativeBuildInputs = [ pkgs.tailwindcss ];
          }
        );

        diff-rs-serve = pkgs.writeShellScriptBin "serve-app" ''
          ${pkgs.python3Minimal}/bin/python3 -m http.server --directory ${diff-rs} 8080
        '';
      in
      {
        checks = {
          inherit diff-rs;

          diff-rs-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          diff-rs-fmt = craneLib.cargoFmt {
            inherit src;
          };
        };

        packages.default = diff-rs;

        apps.default = flake-utils.lib.mkApp {
          drv = diff-rs-serve;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages = [
            pkgs.trunk
          ];
        };
      }
    );
}
