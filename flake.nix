# The core functionality is provided here using flakes. Legacy support for
# `nix-shell` is provided by a wrapper in `shell.nix`.

{
  description = "Bevy development environment";

  inputs = {

    # Version pinning is managed in flake.lock. Upgrading can be done with
    # something like
    #
    #    nix flake lock --update-input nixpkgs

    nixpkgs     .url = "github:nixos/nixpkgs/nixos-23.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils .url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:

    # Option 1: try to support each default system
    flake-utils.lib.eachDefaultSystem # NB Some packages in nixpkgs are not supported on some systems

    # Option 2: try to support selected systems
    # flake-utils.lib.eachSystem ["x86_64-linux" "i686-linux" "aarch64-linux" "x86_64-darwin"]
      (system:

        let pkgs = import nixpkgs {
              inherit system;

              overlays = [
                # ===== Specification of the rust toolchain to be used ====================
                rust-overlay.overlays.default (final: prev:
                  let
                    # If you have a rust-toolchain file for rustup, set `choice =
                    # rust-tcfile` further down to get the customized toolchain
                    # derivation.
                    rust-tcfile  = final.rust-bin.fromRustupToolchainFile ./rust-toolchain;
                    rust-latest  = final.rust-bin.stable .latest      ;
                    rust-beta    = final.rust-bin.beta   .latest      ;
                    rust-nightly = final.rust-bin.nightly."2024-03-03";
                    rust-stable  = final.rust-bin.stable ."1.76.0"    ; # nix flake lock --update-input rust-overlay
                    rust-analyzer-preview-on = date:
                      final.rust-bin.nightly.${date}.default.override {
                        extensions = [ "rust-analyzer-preview" ];
                      };
                  in
                    rec {
                      # The version of the Rust system to be used in buildInputs.
                      # Set `choice` to `rust-<choice>` where `<choice>` is one of
                      #   tcfile / latest / beta / nightly / stable
                      # (see names set above)
                      choice = rust-stable;

                      rust-tools = choice.default.override {
                        # extensions = [];
                        # targets = [ "wasm32-unknown-unknown" ];
                      };
                      rust-analyzer-preview = rust-analyzer-preview-on "2024-03-03";
                      rust-src = rust-stable.rust-src;
                    })
              ];
            };

        in
          {
            devShell = pkgs.mkShell rec {
              name = "bevy-workshop-devshell";

              nativeBuildInputs = [ pkgs.pkg-config ];

              buildInputs = [
                pkgs.rust-tools
                pkgs.rust-analyzer-preview
                pkgs.cargo-nextest
                pkgs.just

                pkgs.udev
                pkgs.alsa-lib
                pkgs.vulkan-loader

                # To use the x11 feature
                pkgs.xorg.libX11
                pkgs.xorg.libXcursor
                pkgs.xorg.libXi
                pkgs.xorg.libXrandr

                # To use the wayland feature
                pkgs.libxkbcommon
                pkgs.wayland

                # pkgs.libGL
                # pkgs.xorg.libxcb

              ];

              packages = with pkgs; [ lolcat eza ];

              shellHook =
                ''
                  export PS1="bevy devshell> "
                  alias foo='cowsay Foo'
                  alias bar='eza -l | lolcat'
                  alias baz='cowsay What is the difference between buildInputs and packages? | lolcat'
                '';

              RUST_SRC_PATH = "${pkgs.rust-src}/lib/rustlib/src/rust/library";
              LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
            };
          }
      );
}
