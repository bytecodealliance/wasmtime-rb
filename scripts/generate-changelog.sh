#! /usr/bin/env bash

set -ex

github_changelog_generator \
  -u bytecodealliance \
  -p wasmtime-rb \
  -t $(gh auth token) \
  --future-release v$(grep VERSION lib/wasmtime/version.rb | head -n 1 | cut -d'"' -f2)
