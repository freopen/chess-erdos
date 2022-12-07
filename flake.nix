{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fup.url = "github:gytis-ivaskevicius/flake-utils-plus";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fup, fenix }@inputs:
    fup.lib.mkFlake {
      inherit self inputs;
      supportedSystems = [ "x86_64-linux" ];
      sharedOverlays = [ fenix.overlay ];

      outputsBuilder = channels: {
        packages.default = channels.nixpkgs.runCommand "chess_erdos"
          {
            src = ./.;
            __noChroot = true;
            nativeBuildInputs = with channels.nixpkgs; [
              binaryen
              cargo-make
              clang
              git
              nodePackages.pnpm
              pkg-config
              protobuf
              wasm-bindgen-cli
              wasm-pack
              (
                with fenix.packages."x86_64-linux";
                combine [
                  minimal.rustc
                  minimal.cargo
                  targets.wasm32-unknown-unknown.latest.rust-std
                ]
              )
            ];
            buildInputs = with channels.nixpkgs; [ openssl curl zstd protobuf ];
            LIBCLANG_PATH = "${channels.nixpkgs.libclang.lib}/lib";
            CURL_CA_BUNDLE = "/etc/ssl/certs/ca-bundle.crt";
            PROTOC = "${channels.nixpkgs.protobuf}/bin/protoc";
          } ''
          cp -R $src/* .
          export CARGO_HOME=$(mktemp -d cargo-home.XXX)

          cargo make release

          mkdir -p $out/bin
          cp target/release-server/chess-erdos $out/bin 
        '';
      };

      nixosModules.default = { config, pkgs, lib, ... }: {
        options.services.chess_erdos = {
          enable = lib.mkEnableOption "chess_erdos";
        };
        config =
          let
            opts = config.services.chess_erdos;
            pkg = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
          in
          lib.mkIf opts.enable {
            users.groups.chess_erdos = { };
            users.users.chess_erdos = {
              isSystemUser = true;
              group = "chess_erdos";
            };
            systemd.services.chess_erdos = {
              wantedBy = [ "multi-user.target" ];
              after = [ "network-online.target" ];
              wants = [ "network-online.target" ];
              path = with pkgs; [ curl zstd ];
              serviceConfig = {
                User = "chess_erdos";
                ExecStart = "${pkg}/bin/chess-erdos";
                WorkingDirectory = "/var/lib/chess_erdos";
                StateDirectory = "chess_erdos";
                StateDirectoryMode = "0700";
                Restart = "always";
              };
            };
          };
      };
    };
}
