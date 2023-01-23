# Contributing

`wasmtime-rb` is a [Bytecode Alliance] project. It follows the Bytecode
Alliance's [Code of Conduct] and [Organizational Code of Conduct].

## Getting started

Install dependencies:

```
bundle install
```

Compile the gem, run the tests & Ruby linter:

```
bundle exec rake
```

## Releasing

1. Bump the `VERSION` in `lib/wasmtime/version.rb`
1. Run `bundle install` to bump the version in `Gemfile.lock`
1. Update the changelog (requires the `github_changelog_generator` gem and being authenticated with `gh`)
  
```
github_changelog_generator \
  -u bytecodealliance \
  -p wasmtime-rb \
  -t $(gh auth token) \
  --future-release v$(grep VERSION lib/wasmtime/version.rb | head -n 1 | cut -d'"' -f2)
```
1. Commit your changes
1. Create a new tag for that release, prefixed with `v` (`git tag v1.0.0`):
  
  ```
  git tag v$(grep VERSION lib/wasmtime/version.rb | head -n 1 | cut -d'"' -f2)
  git push --tags
  ```
1. The release workflow will run and push a new version to RubyGems and create
   a new draft release on GitHub. Edit the release notes if needed, then
   mark the release as published when the release workflow succeeds.


[Bytecode Alliance]: https://bytecodealliance.org/
[Code of Conduct]: https://github.com/bytecodealliance/wasmtime/blob/main/CODE_OF_CONDUCT.md
[Organizational Code of Conduct]: https://github.com/bytecodealliance/wasmtime/blob/main/ORG_CODE_OF_CONDUCT.md
