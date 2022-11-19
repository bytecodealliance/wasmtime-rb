# Contributing

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
1. Create a new tag for that release, prefixed with `v` (`git tag v1.0.0`):
  
  ```
  git tag v$(grep VERSION lib/wasmtime/version.rb | head -n 1 | cut -d'"' -f2)
  git push --tags
  ```
1. The release workflow will run and push a new version to RubyGems and create
   a new draft release on GitHub. Edit the release notes if needed, then
   mark the release as published when the release workflow succeeds.
