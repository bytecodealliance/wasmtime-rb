# Contributing

`wasmtime-rb` is a [Bytecode Alliance] project. It follows the Bytecode
Alliance's [Code of Conduct] and [Organizational Code of Conduct]. Code
contributions must follow the [AI Tool Use Policy].

## Getting started

Install dependencies:

```
bundle install
```

Compile the gem, run the tests & Ruby linter:

```
bundle exec rake
```

## Updating Wasmtime

1. Update the version of `deterministic-wasi-ctx` in `ext/Cargo.toml`
1. Update the `wasmtime-` family of crates to the new version in `ext/Cargo.toml`. Note that this process might involve code changes in case the new version contains public facing API changes.
1. Open a pull request. Don't bump the gem version; merging the pull request
   starts the release (see below).

## Versioning

The gem version matches the version of the `wasmtime` crate it bundles, e.g.
`49.0.2` bundles Wasmtime 49.0.2. The major and minor versions always match.

The patch version may drift. When the gem needs a release without a new
Wasmtime release (e.g. a fix in the gem itself), or when an upstream patch
version is already taken, the gem uses the next unused patch version, e.g. gem
`X.Y.3` bundling Wasmtime `X.Y.2`. The changelog entry of such a release says
which Wasmtime version it bundles.

## Releasing

Releases are prepared by the [Release bump] workflow and published by the
[Release tag] workflow. Nothing is pushed to `main` directly. The workflows
pick the version following [Versioning](#versioning).

### From `main`

1. Merging a pull request that updates the `wasmtime` crate to an unreleased
   version starts the [Release bump] workflow, which pushes a `bump/v<version>`
   branch that bumps the version and updates the changelog. To release without a
   Wasmtime update (e.g. a bug fix in the gem), run the workflow manually with
   branch `main`.
1. Open the pull request from the link in the workflow run's summary. Pull
   requests opened by workflows don't run the required checks, so they have to
   be opened by a maintainer.
1. Review the changelog and merge the pull request. This starts the
   [Release tag] workflow, which tags the version and runs the release
   workflow: it pushes the gem to RubyGems and creates a draft release on
   GitHub.
1. Edit the release notes if needed and publish the draft release.
1. If the release changes the supported versions, update the table in
   [SECURITY.md](SECURITY.md).

### From a release branch

To release a patch for an older version, e.g. `48.0.2` when `main` is on 49:

1. Run the [Release bump] workflow manually with branch `release-<major>`
   (e.g. `release-48`). It creates the branch from the latest `v<major>.*` tag
   if it doesn't exist.
1. Open both pull requests from the links in the workflow run's summary:
   - the release pull request into `release-<major>`: fill in the release notes
     in `CHANGELOG.md`, which the changelog generator can't do for release
     branches;
   - the pull request adding the tag to `exclude-tags` in
     `.github_changelog_generator` on `main`, so that the changelog on `main`
     skips it.
1. Merge both. The [Release tag] workflow doesn't run on its own for release
   branches: run it manually with branch `release-<major>`.
1. Edit the release notes if needed and publish the draft release.

[Bytecode Alliance]: https://bytecodealliance.org/
[Code of Conduct]: https://github.com/bytecodealliance/wasmtime/blob/main/CODE_OF_CONDUCT.md
[Organizational Code of Conduct]: https://github.com/bytecodealliance/wasmtime/blob/main/ORG_CODE_OF_CONDUCT.md
[AI Tool Use Policy]: https://github.com/bytecodealliance/governance/blob/main/AI_TOOL_POLICY.md
[Release bump]: https://github.com/bytecodealliance/wasmtime-rb/actions/workflows/release-bump.yml
[Release tag]: https://github.com/bytecodealliance/wasmtime-rb/actions/workflows/release-tag.yml
