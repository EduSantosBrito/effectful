{
  inputs,
  pkgs,
  ...
}: let
  # Moon from GitHub releases (x86_64-linux). See https://moonrepo.dev/docs/install
  moon = pkgs.stdenv.mkDerivation {
    pname = "moon-cli";
    version = "2.1.3";
    src = pkgs.fetchurl {
      url = "https://github.com/moonrepo/moon/releases/download/v2.1.3/moon_cli-x86_64-unknown-linux-gnu.tar.xz";
      sha256 = "0ir2qh8rifgcmfyb4xyndf9b1yjbn1fzr1gblnj5bnmar99rs60r";
    };
    nativeBuildInputs = [pkgs.autoPatchelfHook];
    buildInputs = [pkgs.stdenv.cc.cc.lib];
    installPhase = ''
      runHook preInstall
      mkdir -p $out/bin
      install -m755 moon $out/bin/moon
      runHook postInstall
    '';
    meta = {
      description = "Moon CLI (moonrepo)";
      homepage = "https://moonrepo.dev";
      license = pkgs.lib.licenses.mit;
      platforms = pkgs.lib.platforms.linux;
    };
  };

  # roam-code: architectural intelligence CLI (https://github.com/Cranot/roam-code)
  roam-code-src = pkgs.fetchFromGitHub {
    owner = "Cranot";
    repo = "roam-code";
    rev = "89bc4d4216dba1977f073323c32eeb7c7221ebe0";
    hash = "sha256-AE1SQaBO/Od1My/nIsH2XQkU2342GIosHf5PJN8NFPg=";
  };
  roam-code = pkgs.python3Packages.buildPythonApplication rec {
    pname = "roam-code";
    version = "10.0.1";
    src = roam-code-src;
    format = "pyproject";
    nativeBuildInputs = with pkgs.python3Packages; [setuptools wheel];
    propagatedBuildInputs = with pkgs.python3Packages; [
      click
      tree-sitter
      tree-sitter-language-pack
      networkx
    ];
    doCheck = false;
  };

  # Fenix nightly matching crates/effect-rs-dylint-rules/rust-toolchain (Dylint / rustc_private).
  fenixPkgs = inputs.fenix.packages.${pkgs.system};
  effectDylintToolchain =
    (fenixPkgs.toolchainOf {
      channel = "nightly";
      date = "2025-09-18";
      sha256 = "13ywswfy0179hymdvbf5w061y2mxc8xd0zik0k31p2z21sc8vv16";
    }).withComponents [
      "cargo"
      "rustc"
      "rust-src"
      "rustc-dev"
      "llvm-tools-preview"
      "clippy"
    ];
in {
  name = "effect-rs";

  dotenv = {
    enable = true;
  };

  packages = with pkgs; [
    cachix
    clippy
    rust-analyzer
    rustc
    perl
    direnv
    prek
    lldb
    cargo-watch
    cargo-audit
    cargo-llvm-cov
    cargo-nextest
    sccache
    mold
    git
    gh
    moon
    roam-code
    actionlint
    alejandra
    beautysh
    biome
    deadnix
    rustfmt
    taplo
    treefmt
    vulnix
    yamlfmt
  ];

  env = {
    CARGO_TERM_COLOR = "always";
    EFFECT_DYLINT_TOOLCHAIN = "${effectDylintToolchain}";
    EFFECT_DYLINT_MOON_PATH_PREFIX = "${toString ./.}/scripts/dylint-rustup-shim:${effectDylintToolchain}/bin";
    DYLINT_RUSTUP_ACTIVE_TOOLCHAIN = "nightly-2025-09-18-${pkgs.stdenv.hostPlatform.config}";
    DYLINT_STABLE_CARGO = "${pkgs.cargo}/bin/cargo";
    MOON_TOOLCHAIN_FORCE_GLOBALS = "rust";
    NEXTEST_NO_TESTS = "pass";
    OPENSSL_NO_VENDOR = "1";
    RUST_LOG = "trace,dylint_driver=info";
  };

  languages.rust = {
    enable = true;
    channel = "stable";
    components = [
      "cargo"
      "clippy"
      "rust-analyzer"
      "rustc"
      "rustfmt"
      "llvm-tools"
    ];
    targets = [];
  };

  scripts = {
    prek-install = {
      exec = ''
        prek install -q --overwrite
      '';
    };

    moon-sync = {
      exec = ''
        moon sync
      '';
    };

    pre-push = {
      exec = ''
        export MOON_TOOLCHAIN_FORCE_GLOBALS=rust
        export MOON_CONCURRENCY=1
        mkdir -p "$DEVENV_ROOT/tmp"
        export TMPDIR="$DEVENV_ROOT/tmp"
        moon run :format :check :build :test :coverage :audit :check-docs
      '';
    };
  };

  enterShell = ''
    mkdir -p "$DEVENV_ROOT/tmp"
    export TMPDIR="$DEVENV_ROOT/tmp"

    prek-install
    moon-sync

    export EFFECT_DYLINT_MOON_PATH="''${EFFECT_DYLINT_MOON_PATH_PREFIX}:''${PATH}"

    _dylint_cli="$DEVENV_ROOT/.devenv/state/dylint-cli"
    _dylint_mark="$_dylint_cli/.installed-v5.0.0-git"
    if [[ ! -f "$_dylint_mark" ]]; then
      echo "devenv: installing cargo-dylint + dylint-link into .devenv/state/dylint-cli (one-time)..." >&2
      rm -rf "$_dylint_cli"
      mkdir -p "$_dylint_cli"
      cargo install --git https://github.com/trailofbits/dylint --tag v5.0.0 --locked --root "$_dylint_cli" cargo-dylint dylint-link
      : >"$_dylint_mark"
    fi
    export PATH="$_dylint_cli/bin:$PATH"

    mkdir -p "$HOME/.cache/sccache"
    chmod 755 "$HOME/.cache/sccache" 2>/dev/null || true
  '';

  git-hooks = {
    default_stages = [
      "pre-push"
      "commit-msg"
    ];

    hooks = {
      pre-commit = {
        enable = true;
        name = "pre-commit";
        entry = "mkdir -p tmp && env TMPDIR=$(pwd)/tmp moon run :format :check :test";
        stages = ["pre-commit"];
        pass_filenames = false;
        always_run = true;
        language = "system";
      };

      pre-push = {
        enable = true;
        name = "pre-push";
        entry = "mkdir -p tmp && env MOON_CONCURRENCY=1 TMPDIR=$(pwd)/tmp moon run :format :check :build :test :coverage :audit :check-docs";
        stages = ["pre-push"];
        pass_filenames = false;
        always_run = true;
        language = "system";
      };

      commitizen = {
        enable = true;
      };
    };
  };
}
