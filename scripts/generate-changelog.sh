#! /usr/bin/env bash

set -e

# github_changelog_generator reads the token from CHANGELOG_GITHUB_TOKEN.
export CHANGELOG_GITHUB_TOKEN=${CHANGELOG_GITHUB_TOKEN:-$(gh auth token)}
repository=${GITHUB_REPOSITORY:-bytecodealliance/wasmtime-rb}

set -x

github_changelog_generator \
  -u "${repository%/*}" \
  -p "${repository#*/}" \
  --future-release "v$(grep VERSION lib/wasmtime/version.rb | head -n 1 | cut -d'"' -f2)"
