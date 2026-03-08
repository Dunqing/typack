#!/usr/bin/env -S just --justfile

set shell := ["bash", "-cu"]

_default:
  @just --list -u

alias r := ready

fmt:
  cargo fmt --all
  pnpm fmt

check:
  cargo check --all-targets --all-features

test:
  cargo test --all-targets --all-features

test-fixture name:
  FIXTURE={{name}} cargo test --test conformance -- --nocapture

lint:
  cargo clippy --all-targets --all-features -- -D warnings

ready:
  just fmt
  just check
  just test
  just lint
