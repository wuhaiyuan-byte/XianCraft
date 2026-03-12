# To learn more about how to use Nix to configure your environment
# see: https://developers.google.com/idx/guides/customize-idx-env
{ pkgs, ... }: {
  # Which nixpkgs channel to use.
  # Using "unstable" to get the latest versions of packages, including Rust.
  channel = "unstable";

  # Use https://search.nixos.org/packages to find packages.
  # We are explicitly installing the latest Rust toolchain available in the unstable channel.
  packages = [
    pkgs.rustc
    pkgs.cargo
    pkgs.rust-analyzer
    pkgs.gcc # A C compiler is often needed for native dependencies in Rust crates.
  ];
}
