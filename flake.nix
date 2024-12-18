{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      inherit (nixpkgs.lib)
        genAttrs
        importTOML
        licenses
        cleanSource
        ;

      eachSystem =
        f:
        genAttrs [
          "aarch64-darwin"
          "aarch64-linux"
          "x86_64-darwin"
          "x86_64-linux"
        ] (system: f nixpkgs.legacyPackages.${system});
    in
    {
      formatter = eachSystem (pkgs: pkgs.nixfmt-rfc-style);

      packages = eachSystem (
        pkgs:
        let
        
        src = cleanSource self
        
          inherit (pkgs)
            rustPlatform
            openssl
            pkg-config
            ;
        in
        {
          default = rustPlatform.buildRustPackage {
            pname = "pay-respects";
            inherit ((importTOML (src + "/core/Cargo.toml")).package) version;

            inherit src;

            cargoLock = {
              lockFile = src + "/Cargo.lock";
            };

            nativeBuildInputs = [ pkg-config ];
            buildInputs = [ openssl ];

            meta = {
              description = "Command suggestions, command-not-found and thefuck replacement written in Rust";
              license = licenses.agpl3Plus;
              homepage = "https://github.com/iffse/pay-respects";
              mainProgram = "pay-respects";
            };
          };
        }
      );
    };
}
