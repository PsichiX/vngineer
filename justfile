list:
  just --list

format:
  cargo fmt --all

build:
  cargo build --all --all-features

test:
  cargo test --all --all-features

clippy:
  cargo clippy --all --all-features

checks:
  just build
  just clippy
  just test

clean:
  find . -name target -type d -exec rm -r {} +
  just remove-lockfiles

remove-lockfiles:
  find . -name Cargo.lock -type f -exec rm {} +

list-outdated:
  cargo outdated -R -w

update:
  cargo update --manifest-path ./engine/Cargo.toml --aggressive
  cargo update --manifest-path ./simpleton/Cargo.toml --aggressive
  cargo update --manifest-path ./runner/Cargo.toml --aggressive

install:
  cargo install --path ./runner

demo:
  cargo run --release --manifest-path ./runner/Cargo.toml -- ./resources/main.vns

publish:
  cargo publish --no-verify --manifest-path ./engine/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./simpleton/Cargo.toml
  sleep 1
  cargo publish --no-verify --manifest-path ./runner/Cargo.toml