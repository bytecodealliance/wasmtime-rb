# Changelog

## [v0.4.1](https://github.com/bytecodealliance/wasmtime-rb/tree/v0.4.1) (2023-01-02)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v0.4.0...v0.4.1)

**Merged pull requests:**

- Fix allocator warning on Ruby 3.2 [\#102](https://github.com/bytecodealliance/wasmtime-rb/pull/102) ([jbourassa](https://github.com/jbourassa))
- Bump rb-sys to v0.9.53 \(Ruby 3.2 support\) [\#101](https://github.com/bytecodealliance/wasmtime-rb/pull/101) ([jbourassa](https://github.com/jbourassa))
- Bump cap-std from 1.0.2 to 1.0.3 [\#99](https://github.com/bytecodealliance/wasmtime-rb/pull/99) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump standard from 1.19.1 to 1.20.0 [\#98](https://github.com/bytecodealliance/wasmtime-rb/pull/98) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.4.0](https://github.com/bytecodealliance/wasmtime-rb/tree/v0.4.0) (2022-12-21)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v0.3.0...v0.4.0)

**Closed issues:**

- Wasmtime::WasiCtxBuilder not available from Ruby [\#80](https://github.com/bytecodealliance/wasmtime-rb/issues/80)
- Can't use git source with bundler [\#51](https://github.com/bytecodealliance/wasmtime-rb/issues/51)
- Support fuel [\#25](https://github.com/bytecodealliance/wasmtime-rb/issues/25)
- Support Epoch interruption [\#23](https://github.com/bytecodealliance/wasmtime-rb/issues/23)
- Missing engine Config [\#22](https://github.com/bytecodealliance/wasmtime-rb/issues/22)
- Support WASI [\#21](https://github.com/bytecodealliance/wasmtime-rb/issues/21)
- Support Tables [\#20](https://github.com/bytecodealliance/wasmtime-rb/issues/20)
- Support Globals [\#19](https://github.com/bytecodealliance/wasmtime-rb/issues/19)
- Ideas / Feedback [\#10](https://github.com/bytecodealliance/wasmtime-rb/issues/10)

**Merged pull requests:**

- Update Wasmtime to v4.0.0 [\#95](https://github.com/bytecodealliance/wasmtime-rb/pull/95) ([jbourassa](https://github.com/jbourassa))
- Speed up func calls [\#94](https://github.com/bytecodealliance/wasmtime-rb/pull/94) ([jbourassa](https://github.com/jbourassa))
- Further simplify error handling [\#93](https://github.com/bytecodealliance/wasmtime-rb/pull/93) ([jbourassa](https://github.com/jbourassa))
- Improve error handling [\#91](https://github.com/bytecodealliance/wasmtime-rb/pull/91) ([jbourassa](https://github.com/jbourassa))
- `README.md` & `CONTRIBUTING.md` changes [\#90](https://github.com/bytecodealliance/wasmtime-rb/pull/90) ([jbourassa](https://github.com/jbourassa))
- Engine config [\#89](https://github.com/bytecodealliance/wasmtime-rb/pull/89) ([jbourassa](https://github.com/jbourassa))
- Bump anyhow from 1.0.66 to 1.0.68 [\#88](https://github.com/bytecodealliance/wasmtime-rb/pull/88) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump rake-compiler from 1.2.0 to 1.2.1 [\#86](https://github.com/bytecodealliance/wasmtime-rb/pull/86) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bumb rb-sys to v0.9.52 [\#85](https://github.com/bytecodealliance/wasmtime-rb/pull/85) ([jbourassa](https://github.com/jbourassa))
- More examples [\#84](https://github.com/bytecodealliance/wasmtime-rb/pull/84) ([jbourassa](https://github.com/jbourassa))
- Remove 2 lingering `FuncType`s [\#82](https://github.com/bytecodealliance/wasmtime-rb/pull/82) ([jbourassa](https://github.com/jbourassa))
- Merge`*Type` on their respective class \(`Type`, `Memory`, ...\) [\#81](https://github.com/bytecodealliance/wasmtime-rb/pull/81) ([jbourassa](https://github.com/jbourassa))
- Update rb\_sys crate & gem to 0.9.50 [\#79](https://github.com/bytecodealliance/wasmtime-rb/pull/79) ([jbourassa](https://github.com/jbourassa))
- Bump rb-sys from 0.9.46 to 0.9.48 [\#78](https://github.com/bytecodealliance/wasmtime-rb/pull/78) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump rb\_sys from 0.9.46 to 0.9.48 [\#77](https://github.com/bytecodealliance/wasmtime-rb/pull/77) ([dependabot[bot]](https://github.com/apps/dependabot))
- Implement Engine epoch timers with Tokio [\#76](https://github.com/bytecodealliance/wasmtime-rb/pull/76) ([jbourassa](https://github.com/jbourassa))
- Attempt to fix memcheck [\#75](https://github.com/bytecodealliance/wasmtime-rb/pull/75) ([jbourassa](https://github.com/jbourassa))
- Use `gc::mark_slice` where possible [\#74](https://github.com/bytecodealliance/wasmtime-rb/pull/74) ([jbourassa](https://github.com/jbourassa))
- Add global support [\#73](https://github.com/bytecodealliance/wasmtime-rb/pull/73) ([jbourassa](https://github.com/jbourassa))
- Add benchmarks [\#72](https://github.com/bytecodealliance/wasmtime-rb/pull/72) ([jbourassa](https://github.com/jbourassa))
- Minor fixes for table [\#71](https://github.com/bytecodealliance/wasmtime-rb/pull/71) ([jbourassa](https://github.com/jbourassa))
- Add table support [\#70](https://github.com/bytecodealliance/wasmtime-rb/pull/70) ([jbourassa](https://github.com/jbourassa))
- Add env configurations for `dev` and `release` [\#69](https://github.com/bytecodealliance/wasmtime-rb/pull/69) ([ianks](https://github.com/ianks))
- Make `wasmtime-rb` usable as a Rust crate [\#68](https://github.com/bytecodealliance/wasmtime-rb/pull/68) ([ianks](https://github.com/ianks))
- Bump wasmtime to 3.0.1 [\#67](https://github.com/bytecodealliance/wasmtime-rb/pull/67) ([jbourassa](https://github.com/jbourassa))
- Bump standard from 1.18.1 to 1.19.1 [\#66](https://github.com/bytecodealliance/wasmtime-rb/pull/66) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump yard-rustdoc from 0.3.0 to 0.3.2 [\#64](https://github.com/bytecodealliance/wasmtime-rb/pull/64) ([dependabot[bot]](https://github.com/apps/dependabot))
- Add rake task to run examples [\#58](https://github.com/bytecodealliance/wasmtime-rb/pull/58) ([ianks](https://github.com/ianks))
- Use `magnus` release from crates.io [\#57](https://github.com/bytecodealliance/wasmtime-rb/pull/57) ([ianks](https://github.com/ianks))
- Add `mswin` to CI matrix [\#56](https://github.com/bytecodealliance/wasmtime-rb/pull/56) ([ianks](https://github.com/ianks))
- Add info about precompiled gems to readme [\#55](https://github.com/bytecodealliance/wasmtime-rb/pull/55) ([ianks](https://github.com/ianks))
- Move native ext SO to `wasmtime` dir [\#53](https://github.com/bytecodealliance/wasmtime-rb/pull/53) ([jbourassa](https://github.com/jbourassa))
- Add `Wasmtime.wat2wasm` [\#52](https://github.com/bytecodealliance/wasmtime-rb/pull/52) ([jbourassa](https://github.com/jbourassa))
- Add fuel [\#50](https://github.com/bytecodealliance/wasmtime-rb/pull/50) ([jbourassa](https://github.com/jbourassa))
- Add custom task to build the source gem [\#49](https://github.com/bytecodealliance/wasmtime-rb/pull/49) ([ianks](https://github.com/ianks))
- Refactor specs [\#48](https://github.com/bytecodealliance/wasmtime-rb/pull/48) ([jbourassa](https://github.com/jbourassa))
- Set min `rb_sys` gem to v0.9.44 [\#47](https://github.com/bytecodealliance/wasmtime-rb/pull/47) ([ianks](https://github.com/ianks))
- Bump standard from 1.18.0 to 1.18.1 [\#46](https://github.com/bytecodealliance/wasmtime-rb/pull/46) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump rb\_sys from 0.9.41 to 0.9.44 [\#45](https://github.com/bytecodealliance/wasmtime-rb/pull/45) ([dependabot[bot]](https://github.com/apps/dependabot))
- Support WASI [\#43](https://github.com/bytecodealliance/wasmtime-rb/pull/43) ([jbourassa](https://github.com/jbourassa))
- Fix build "smoke test" step from Wasmtime 3 [\#42](https://github.com/bytecodealliance/wasmtime-rb/pull/42) ([jbourassa](https://github.com/jbourassa))
- Wasmtime 3.0.0 [\#41](https://github.com/bytecodealliance/wasmtime-rb/pull/41) ([jbourassa](https://github.com/jbourassa))
- Examples and README [\#40](https://github.com/bytecodealliance/wasmtime-rb/pull/40) ([jbourassa](https://github.com/jbourassa))
- Implement `Module.from_file` [\#39](https://github.com/bytecodealliance/wasmtime-rb/pull/39) ([jbourassa](https://github.com/jbourassa))
- Bump rb-sys from 0.9.39 to 0.9.44 [\#38](https://github.com/bytecodealliance/wasmtime-rb/pull/38) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump magnus from `aa79114` to `d6f4152` [\#37](https://github.com/bytecodealliance/wasmtime-rb/pull/37) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump standard from 1.17.0 to 1.18.0 [\#36](https://github.com/bytecodealliance/wasmtime-rb/pull/36) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump k1LoW/github-script-ruby from 1 to 2 [\#35](https://github.com/bytecodealliance/wasmtime-rb/pull/35) ([dependabot[bot]](https://github.com/apps/dependabot))
- Add support for funcref [\#34](https://github.com/bytecodealliance/wasmtime-rb/pull/34) ([jbourassa](https://github.com/jbourassa))
- Limit workflow concurrency [\#33](https://github.com/bytecodealliance/wasmtime-rb/pull/33) ([jbourassa](https://github.com/jbourassa))
- Prepare for prerelease [\#32](https://github.com/bytecodealliance/wasmtime-rb/pull/32) ([jbourassa](https://github.com/jbourassa))
- Implement Trap [\#30](https://github.com/bytecodealliance/wasmtime-rb/pull/30) ([jbourassa](https://github.com/jbourassa))
- Publish documentation automatically [\#29](https://github.com/bytecodealliance/wasmtime-rb/pull/29) ([jbourassa](https://github.com/jbourassa))
- Improve the docs after \#14 [\#28](https://github.com/bytecodealliance/wasmtime-rb/pull/28) ([jbourassa](https://github.com/jbourassa))
- Add new `mem:check` task to run Valgrind in CI [\#26](https://github.com/bytecodealliance/wasmtime-rb/pull/26) ([ianks](https://github.com/ianks))
- Bump magnus from `1348da5` to `aa79114` [\#18](https://github.com/bytecodealliance/wasmtime-rb/pull/18) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump standard from 1.16.1 to 1.17.0 [\#17](https://github.com/bytecodealliance/wasmtime-rb/pull/17) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump ruby-lsp from 0.3.5 to 0.3.6 [\#16](https://github.com/bytecodealliance/wasmtime-rb/pull/16) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump rb\_sys from 0.9.39 to 0.9.41 [\#15](https://github.com/bytecodealliance/wasmtime-rb/pull/15) ([dependabot[bot]](https://github.com/apps/dependabot))
- Add  new `Wasmtime::Extern` class [\#14](https://github.com/bytecodealliance/wasmtime-rb/pull/14) ([ianks](https://github.com/ianks))
- YARD doc generation [\#13](https://github.com/bytecodealliance/wasmtime-rb/pull/13) ([jbourassa](https://github.com/jbourassa))
- Update Wasmtime to 2.0.2 [\#12](https://github.com/bytecodealliance/wasmtime-rb/pull/12) ([jbourassa](https://github.com/jbourassa))
- Add precompiled gems for `mingw` [\#11](https://github.com/bytecodealliance/wasmtime-rb/pull/11) ([ianks](https://github.com/ianks))
- Bump rb\_sys from 0.9.35 to 0.9.37 [\#9](https://github.com/bytecodealliance/wasmtime-rb/pull/9) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump wasmtime from 2.0.0 to 2.0.1 [\#7](https://github.com/bytecodealliance/wasmtime-rb/pull/7) ([dependabot[bot]](https://github.com/apps/dependabot))
- Fix dependabot config for cargo [\#6](https://github.com/bytecodealliance/wasmtime-rb/pull/6) ([ianks](https://github.com/ianks))
- Add dependabot and ruby-lsp [\#5](https://github.com/bytecodealliance/wasmtime-rb/pull/5) ([ianks](https://github.com/ianks))
- Add `Module.deserialize_file` [\#4](https://github.com/bytecodealliance/wasmtime-rb/pull/4) ([ianks](https://github.com/ianks))
- Add hard-mode CI step with `GC.stress` [\#3](https://github.com/bytecodealliance/wasmtime-rb/pull/3) ([ianks](https://github.com/ianks))
- Implement `Caller#export` [\#2](https://github.com/bytecodealliance/wasmtime-rb/pull/2) ([jbourassa](https://github.com/jbourassa))
- Setup initial cross compilation workflow [\#1](https://github.com/bytecodealliance/wasmtime-rb/pull/1) ([ianks](https://github.com/ianks))

## [v0.3.0](https://github.com/bytecodealliance/wasmtime-rb/tree/v0.3.0) (2022-11-24)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/7d6150fcfaca5801c755a2bf6b425696e4aad3e3...v0.3.0)

**Closed issues:**

- Raise `Trap` exception for Wasm traps [\#24](https://github.com/bytecodealliance/wasmtime-rb/issues/24)



\* *This Changelog was automatically generated by [github_changelog_generator](https://github.com/github-changelog-generator/github-changelog-generator)*
