# Security Policy

## Reporting a vulnerability

Please report vulnerabilities privately through
[GitHub Security Advisories](https://github.com/bytecodealliance/wasmtime-rb/security/advisories/new).
Do not open a public issue.

Vulnerabilities in Wasmtime itself (rather than in these Ruby bindings) should
be reported to the [Wasmtime project](https://github.com/bytecodealliance/wasmtime/blob/main/SECURITY.md).

## Supported versions

`wasmtime-rb` follows [Wasmtime's release support policy](https://docs.wasmtime.dev/stability-release.html).
Security fixes are released for:

- The latest release.
- The previous release.
- Wasmtime LTS lines (versions divisible by 12) for as long as Wasmtime
  supports them, if `wasmtime-rb` published a release for that line.

| Version | Supported           |
| ------- | ------------------- |
| 49.x    | Yes (latest)        |
| 48.x    | Yes (previous, LTS) |
| 36.x    | Yes (LTS)           |
| Others  | No                  |

## Security releases

Fixes are developed privately and released on the same day for every affected
supported version, as patch releases from `main` or from a `release-<major>`
branch (see [Releasing](CONTRIBUTING.md#releasing)). The advisory is published
once the patched gems are available on RubyGems. Maintainers follow the
[vulnerability runbook](docs/security-vulnerability-runbook.md).

A security release may bump only the patch version, so the gem version can
differ from the Wasmtime version it bundles; see
[Versioning](CONTRIBUTING.md#versioning).
