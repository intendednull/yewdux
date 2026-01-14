{
  pkgs ? let
    lock = (builtins.fromJSON (builtins.readFile ../flake.lock)).nodes.nixpkgs.locked;
    nixpkgs = fetchTarball {
      url = "https://github.com/nixos/nixpkgs/archive/${lock.rev}.tar.gz";
      sha256 = lock.narHash;
    };
  in
    import nixpkgs {overlays = [];},
  ...
}: let
  lib = pkgs.lib;
  manifest = (pkgs.lib.importTOML ./crates/yewdux/Cargo.toml).package;
in
  pkgs.rustPlatform.buildRustPackage {
    pname = manifest.name;
    version = manifest.version;
    src = pkgs.lib.cleanSource ./.;

    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    nativeBuildInputs = with pkgs; [
      binaryen
      llvmPackages.bintools-unwrapped
      tailwindcss_4
      trunk
      # needs to match with wasm-bindgen version in upstreams Cargo.lock
      wasm-bindgen-cli_0_2_93
    ];

    buildPhase = ''
      trunk build --offline --frozen --release
    '';

    installPhase = ''
      cd dist
      mkdir -p $out
      mv * $out
    '';
  }
