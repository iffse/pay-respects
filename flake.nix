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
          cargoPackage = (importTOML (src + "/core/Cargo.toml")).package;

          src = cleanSource self;

          inherit (pkgs)
            rustPlatform
            openssl
            pkg-config
            ;
        in
        {
          default = rustPlatform.buildRustPackage {
            pname = cargoPackage.name;
            inherit (cargoPackage) version;

            inherit src;

            cargoLock = {
              lockFile = src + "/Cargo.lock";
            };

            nativeBuildInputs = [ pkg-config ];
            buildInputs = [ openssl ];

            meta = {
              inherit (cargoPackage) description homepage;
              license = licenses.agpl3Plus;
              mainProgram = "pay-respects";
            };
          };
        }
      );
    };
}
