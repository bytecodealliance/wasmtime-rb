# Changelog

## [v17.0.1](https://github.com/bytecodealliance/wasmtime-rb/tree/v17.0.1) (2024-02-12)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v17.0.0...v17.0.1)

**Merged pull requests:**

- chore: Disable Ruby head temporarily [\#296](https://github.com/bytecodealliance/wasmtime-rb/pull/296) ([saulecabrera](https://github.com/saulecabrera))
- upgrade wasmtime to 17.0.1 [\#295](https://github.com/bytecodealliance/wasmtime-rb/pull/295) ([glebpom](https://github.com/glebpom))

## [v17.0.0](https://github.com/bytecodealliance/wasmtime-rb/tree/v17.0.0) (2024-01-30)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v16.0.0...v17.0.0)

**Merged pull requests:**

- \[ci skip\] chore: Update release workflow to use download-artifact@v3 [\#288](https://github.com/bytecodealliance/wasmtime-rb/pull/288) ([saulecabrera](https://github.com/saulecabrera))
- chore: Update to wasmtime v17 [\#285](https://github.com/bytecodealliance/wasmtime-rb/pull/285) ([saulecabrera](https://github.com/saulecabrera))
- chore: Bump download artifact in publish doc workflow [\#284](https://github.com/bytecodealliance/wasmtime-rb/pull/284) ([saulecabrera](https://github.com/saulecabrera))
- Expose Wasi Context to Ruby [\#282](https://github.com/bytecodealliance/wasmtime-rb/pull/282) ([cameronbarker](https://github.com/cameronbarker))
- Add support for resource limits on `Wasmtime::Store` [\#281](https://github.com/bytecodealliance/wasmtime-rb/pull/281) ([ryanische](https://github.com/ryanische))
- Unlock the GVL when compiling WASM code [\#277](https://github.com/bytecodealliance/wasmtime-rb/pull/277) ([ianks](https://github.com/ianks))
- chore\(deps\): bump actions/download-artifact from 3 to 4 [\#257](https://github.com/bytecodealliance/wasmtime-rb/pull/257) ([dependabot[bot]](https://github.com/apps/dependabot))
- chore\(deps\): bump actions/upload-artifact from 3 to 4 [\#256](https://github.com/bytecodealliance/wasmtime-rb/pull/256) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v16.0.0](https://github.com/bytecodealliance/wasmtime-rb/tree/v16.0.0) (2024-01-11)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v15.0.1...v16.0.0)

**Merged pull requests:**

- chore: Update to wasmtime v16 [\#280](https://github.com/bytecodealliance/wasmtime-rb/pull/280) ([saulecabrera](https://github.com/saulecabrera))

## [v15.0.1](https://github.com/bytecodealliance/wasmtime-rb/tree/v15.0.1) (2024-01-11)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v15.0.0...v15.0.1)

**Merged pull requests:**

- chore: Update to wasmtime v15.0.1 [\#278](https://github.com/bytecodealliance/wasmtime-rb/pull/278) ([saulecabrera](https://github.com/saulecabrera))

## [v15.0.0](https://github.com/bytecodealliance/wasmtime-rb/tree/v15.0.0) (2024-01-09)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v14.0.4...v15.0.0)

**Closed issues:**

- Upgrade to magnus v0.6.1 [\#215](https://github.com/bytecodealliance/wasmtime-rb/issues/215)

**Merged pull requests:**

- Fix occasional Func call params conversion error [\#274](https://github.com/bytecodealliance/wasmtime-rb/pull/274) ([matsadler](https://github.com/matsadler))
- Update dependencies [\#273](https://github.com/bytecodealliance/wasmtime-rb/pull/273) ([jbourassa](https://github.com/jbourassa))
- Fix gemspec for 3.3 compat [\#272](https://github.com/bytecodealliance/wasmtime-rb/pull/272) ([jbourassa](https://github.com/jbourassa))
- Add support for precompiled binaries on Ruby 3.3 [\#270](https://github.com/bytecodealliance/wasmtime-rb/pull/270) ([ianks](https://github.com/ianks))
- chore: Update to `wasmtime` 15 [\#266](https://github.com/bytecodealliance/wasmtime-rb/pull/266) ([saulecabrera](https://github.com/saulecabrera))
- Allow perfmap as a profiling config [\#255](https://github.com/bytecodealliance/wasmtime-rb/pull/255) ([Maaarcocr](https://github.com/Maaarcocr))
- chore\(deps\): bump rb-sys from 0.9.82 to 0.9.83 [\#252](https://github.com/bytecodealliance/wasmtime-rb/pull/252) ([dependabot[bot]](https://github.com/apps/dependabot))
- chore\(deps\): bump rb\_sys from 0.9.82 to 0.9.83 [\#249](https://github.com/bytecodealliance/wasmtime-rb/pull/249) ([dependabot[bot]](https://github.com/apps/dependabot))
- chore\(deps-dev\): bump standard from 1.31.2 to 1.32.0 [\#248](https://github.com/bytecodealliance/wasmtime-rb/pull/248) ([dependabot[bot]](https://github.com/apps/dependabot))
- Allow `generate_address_map` config in `Engine` [\#247](https://github.com/bytecodealliance/wasmtime-rb/pull/247) ([Maaarcocr](https://github.com/Maaarcocr))
- Upgrade Magnus to 0.6 [\#205](https://github.com/bytecodealliance/wasmtime-rb/pull/205) ([matsadler](https://github.com/matsadler))

## [v14.0.4](https://github.com/bytecodealliance/wasmtime-rb/tree/v14.0.4) (2023-11-09)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v14.0.3...v14.0.4)

**Merged pull requests:**

- chore: Update to `wasmtime` 14.0.4 [\#246](https://github.com/bytecodealliance/wasmtime-rb/pull/246) ([saulecabrera](https://github.com/saulecabrera))
- chore\(deps\): bump wat from 1.0.77 to 1.0.79 [\#245](https://github.com/bytecodealliance/wasmtime-rb/pull/245) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump async-timer from 1.0.0-beta.10 to 1.0.0-beta.11 [\#243](https://github.com/bytecodealliance/wasmtime-rb/pull/243) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v14.0.3](https://github.com/bytecodealliance/wasmtime-rb/tree/v14.0.3) (2023-11-07)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v14.0.2...v14.0.3)

**Merged pull requests:**

- chore: Update to `wasmtimev14.0.3` [\#244](https://github.com/bytecodealliance/wasmtime-rb/pull/244) ([saulecabrera](https://github.com/saulecabrera))
- Bump rake from 13.0.6 to 13.1.0 [\#241](https://github.com/bytecodealliance/wasmtime-rb/pull/241) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump standard from 1.31.1 to 1.31.2 [\#240](https://github.com/bytecodealliance/wasmtime-rb/pull/240) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v14.0.2](https://github.com/bytecodealliance/wasmtime-rb/tree/v14.0.2) (2023-11-01)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v14.0.1...v14.0.2)

**Merged pull requests:**

- chore: Update to `wasmtime` 14.0.2 [\#239](https://github.com/bytecodealliance/wasmtime-rb/pull/239) ([saulecabrera](https://github.com/saulecabrera))
- Add support for using Winch as a compiler strategy [\#238](https://github.com/bytecodealliance/wasmtime-rb/pull/238) ([jeffcharles](https://github.com/jeffcharles))

## [v14.0.1](https://github.com/bytecodealliance/wasmtime-rb/tree/v14.0.1) (2023-10-30)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v14.0.0...v14.0.1)

**Merged pull requests:**

- chore: Update to `wasmtime` 14.0.1 [\#237](https://github.com/bytecodealliance/wasmtime-rb/pull/237) ([saulecabrera](https://github.com/saulecabrera))

## [v14.0.0](https://github.com/bytecodealliance/wasmtime-rb/tree/v14.0.0) (2023-10-26)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v13.0.0...v14.0.0)

**Merged pull requests:**

- chore: Update to wasmtime 14 [\#236](https://github.com/bytecodealliance/wasmtime-rb/pull/236) ([saulecabrera](https://github.com/saulecabrera))
- chore: Update wat [\#235](https://github.com/bytecodealliance/wasmtime-rb/pull/235) ([saulecabrera](https://github.com/saulecabrera))
- Bump actions/checkout from 3 to 4 [\#234](https://github.com/bytecodealliance/wasmtime-rb/pull/234) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump rb-sys from 0.9.81 to 0.9.82 [\#230](https://github.com/bytecodealliance/wasmtime-rb/pull/230) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump rb\_sys from 0.9.81 to 0.9.82 [\#228](https://github.com/bytecodealliance/wasmtime-rb/pull/228) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump standard from 1.31.0 to 1.31.1 [\#227](https://github.com/bytecodealliance/wasmtime-rb/pull/227) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v13.0.0](https://github.com/bytecodealliance/wasmtime-rb/tree/v13.0.0) (2023-10-02)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v12.0.1...v13.0.0)

**Merged pull requests:**

- chore: Update to wasmtime 13.0.0 [\#225](https://github.com/bytecodealliance/wasmtime-rb/pull/225) ([saulecabrera](https://github.com/saulecabrera))

## [v12.0.1](https://github.com/bytecodealliance/wasmtime-rb/tree/v12.0.1) (2023-09-07)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v12.0.0...v12.0.1)

**Merged pull requests:**

- chore: Wasmtime v12.0.1 [\#224](https://github.com/bytecodealliance/wasmtime-rb/pull/224) ([saulecabrera](https://github.com/saulecabrera))
- Bump standard from 1.30.1 to 1.31.0 [\#222](https://github.com/bytecodealliance/wasmtime-rb/pull/222) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v12.0.0](https://github.com/bytecodealliance/wasmtime-rb/tree/v12.0.0) (2023-08-31)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v11.0.0...v12.0.0)

**Merged pull requests:**

- Update wasmtime@12 [\#221](https://github.com/bytecodealliance/wasmtime-rb/pull/221) ([saulecabrera](https://github.com/saulecabrera))

## [v11.0.0](https://github.com/bytecodealliance/wasmtime-rb/tree/v11.0.0) (2023-08-30)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v10.0.1...v11.0.0)

**Merged pull requests:**

- Update wasmtime@11.0.0 [\#220](https://github.com/bytecodealliance/wasmtime-rb/pull/220) ([saulecabrera](https://github.com/saulecabrera))

## [v10.0.1](https://github.com/bytecodealliance/wasmtime-rb/tree/v10.0.1) (2023-08-30)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v10.0.0...v10.0.1)

**Merged pull requests:**

- Update to wasmtime@10.0.1 [\#219](https://github.com/bytecodealliance/wasmtime-rb/pull/219) ([saulecabrera](https://github.com/saulecabrera))

## [v10.0.0](https://github.com/bytecodealliance/wasmtime-rb/tree/v10.0.0) (2023-08-28)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v9.0.4...v10.0.0)

**Merged pull requests:**

- chore: Update to wasmtime@10 [\#218](https://github.com/bytecodealliance/wasmtime-rb/pull/218) ([saulecabrera](https://github.com/saulecabrera))

## [v9.0.4](https://github.com/bytecodealliance/wasmtime-rb/tree/v9.0.4) (2023-08-22)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v9.0.1...v9.0.4)

**Merged pull requests:**

- Bump rb\_sys from 0.9.78 to 0.9.81 [\#214](https://github.com/bytecodealliance/wasmtime-rb/pull/214) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump rb-sys from 0.9.78 to 0.9.81 [\#213](https://github.com/bytecodealliance/wasmtime-rb/pull/213) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump rake-compiler from 1.2.1 to 1.2.5 [\#212](https://github.com/bytecodealliance/wasmtime-rb/pull/212) ([dependabot[bot]](https://github.com/apps/dependabot))
- Address Ruby dep issues and bump to `v9.0.4` [\#211](https://github.com/bytecodealliance/wasmtime-rb/pull/211) ([ianks](https://github.com/ianks))
- Bump wat from 1.0.64 to 1.0.69 [\#209](https://github.com/bytecodealliance/wasmtime-rb/pull/209) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump standard from 1.28.2 to 1.30.1 [\#206](https://github.com/bytecodealliance/wasmtime-rb/pull/206) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump tokio from 1.28.1 to 1.29.1 [\#201](https://github.com/bytecodealliance/wasmtime-rb/pull/201) ([dependabot[bot]](https://github.com/apps/dependabot))
- Pin nightly and use the same magnus version [\#197](https://github.com/bytecodealliance/wasmtime-rb/pull/197) ([saulecabrera](https://github.com/saulecabrera))

## [v9.0.1](https://github.com/bytecodealliance/wasmtime-rb/tree/v9.0.1) (2023-05-23)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v8.0.0...v9.0.1)

**Closed issues:**

- Add documentation about fork safety with `Wasmtime::Engine` [\#174](https://github.com/bytecodealliance/wasmtime-rb/issues/174)

**Merged pull requests:**

- Upgrade Wasmtime 9 [\#191](https://github.com/bytecodealliance/wasmtime-rb/pull/191) ([ianks](https://github.com/ianks))
- Bump rb-sys from 0.9.72 to 0.9.77 [\#190](https://github.com/bytecodealliance/wasmtime-rb/pull/190) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump rb\_sys from 0.9.70 to 0.9.77 [\#189](https://github.com/bytecodealliance/wasmtime-rb/pull/189) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump tokio from 1.27.0 to 1.28.1 [\#188](https://github.com/bytecodealliance/wasmtime-rb/pull/188) ([dependabot[bot]](https://github.com/apps/dependabot))
- Report memory usage to the Ruby GC [\#187](https://github.com/bytecodealliance/wasmtime-rb/pull/187) ([ianks](https://github.com/ianks))
- Bump standard from 1.25.3 to 1.28.0 [\#186](https://github.com/bytecodealliance/wasmtime-rb/pull/186) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump yard from 0.9.28 to 0.9.34 [\#185](https://github.com/bytecodealliance/wasmtime-rb/pull/185) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump wat from 1.0.62 to 1.0.63 [\#184](https://github.com/bytecodealliance/wasmtime-rb/pull/184) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump anyhow from 1.0.70 to 1.0.71 [\#179](https://github.com/bytecodealliance/wasmtime-rb/pull/179) ([dependabot[bot]](https://github.com/apps/dependabot))
- Test that funcrefs aren't used across stores [\#178](https://github.com/bytecodealliance/wasmtime-rb/pull/178) ([jbourassa](https://github.com/jbourassa))
- Add docs for fork safety [\#177](https://github.com/bytecodealliance/wasmtime-rb/pull/177) ([ianks](https://github.com/ianks))

## [v8.0.0](https://github.com/bytecodealliance/wasmtime-rb/tree/v8.0.0) (2023-04-25)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v7.0.0...v8.0.0)

**Merged pull requests:**

- Release v8.0.0 [\#176](https://github.com/bytecodealliance/wasmtime-rb/pull/176) ([jbourassa](https://github.com/jbourassa))
- Remove warning from Ractor spec [\#175](https://github.com/bytecodealliance/wasmtime-rb/pull/175) ([jbourassa](https://github.com/jbourassa))
- Wasmtime 8.0 [\#173](https://github.com/bytecodealliance/wasmtime-rb/pull/173) ([jbourassa](https://github.com/jbourassa))
- Forward compatibility with future version of Magnus [\#172](https://github.com/bytecodealliance/wasmtime-rb/pull/172) ([matsadler](https://github.com/matsadler))
- Add "insanity" specs [\#171](https://github.com/bytecodealliance/wasmtime-rb/pull/171) ([ianks](https://github.com/ianks))
- Make `Param` be `Copy` and mark exceptions raised from Ruby [\#158](https://github.com/bytecodealliance/wasmtime-rb/pull/158) ([ianks](https://github.com/ianks))

## [v7.0.0](https://github.com/bytecodealliance/wasmtime-rb/tree/v7.0.0) (2023-04-06)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v6.0.1...v7.0.0)

**Closed issues:**

- Segfault on macOS in Func error handling [\#156](https://github.com/bytecodealliance/wasmtime-rb/issues/156)

**Merged pull requests:**

- Drop support for Ruby 2.7 [\#170](https://github.com/bytecodealliance/wasmtime-rb/pull/170) ([jbourassa](https://github.com/jbourassa))
- Default to Ruby 3.2 in all workflows [\#168](https://github.com/bytecodealliance/wasmtime-rb/pull/168) ([jbourassa](https://github.com/jbourassa))
- Update deps in `examples/rust-crate` [\#167](https://github.com/bytecodealliance/wasmtime-rb/pull/167) ([jbourassa](https://github.com/jbourassa))
- chore\(pkg\): Fix permissions of vendored files [\#166](https://github.com/bytecodealliance/wasmtime-rb/pull/166) ([saulecabrera](https://github.com/saulecabrera))
- chore\(ext\): Update extension crate authors [\#165](https://github.com/bytecodealliance/wasmtime-rb/pull/165) ([saulecabrera](https://github.com/saulecabrera))
- Release v7.0.0 [\#164](https://github.com/bytecodealliance/wasmtime-rb/pull/164) ([jbourassa](https://github.com/jbourassa))
- Wasmtime 7.0 [\#163](https://github.com/bytecodealliance/wasmtime-rb/pull/163) ([jbourassa](https://github.com/jbourassa))
- Fix potential GC bug when expiring the caller [\#162](https://github.com/bytecodealliance/wasmtime-rb/pull/162) ([jbourassa](https://github.com/jbourassa))
- Allowing making Module and Engine Ractor shareable [\#161](https://github.com/bytecodealliance/wasmtime-rb/pull/161) ([macournoyer](https://github.com/macournoyer))
- Mark the Gem as Ractor safe [\#160](https://github.com/bytecodealliance/wasmtime-rb/pull/160) ([macournoyer](https://github.com/macournoyer))
- Use Magnus macros to implement TypedData trait [\#159](https://github.com/bytecodealliance/wasmtime-rb/pull/159) ([matsadler](https://github.com/matsadler))
- Add support for configuring the engine's `target` [\#157](https://github.com/bytecodealliance/wasmtime-rb/pull/157) ([ianks](https://github.com/ianks))

## [v6.0.1](https://github.com/bytecodealliance/wasmtime-rb/tree/v6.0.1) (2023-03-13)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v6.0.0...v6.0.1)

**Merged pull requests:**

- Bump Wasmtime to 6.0.1 [\#155](https://github.com/bytecodealliance/wasmtime-rb/pull/155) ([jbourassa](https://github.com/jbourassa))

## [v6.0.0](https://github.com/bytecodealliance/wasmtime-rb/tree/v6.0.0) (2023-03-06)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v5.0.0...v6.0.0)

**Closed issues:**

- Precompiled gem for Ruby 3.2 [\#103](https://github.com/bytecodealliance/wasmtime-rb/issues/103)

**Merged pull requests:**

- Update Wasmtime to 6.0 [\#152](https://github.com/bytecodealliance/wasmtime-rb/pull/152) ([jbourassa](https://github.com/jbourassa))
- Fix rb-sys build warning [\#151](https://github.com/bytecodealliance/wasmtime-rb/pull/151) ([jbourassa](https://github.com/jbourassa))
- Reduce dependabot frequency [\#150](https://github.com/bytecodealliance/wasmtime-rb/pull/150) ([sandstrom](https://github.com/sandstrom))
- Bump rb-sys from 0.9.64 to 0.9.65 [\#149](https://github.com/bytecodealliance/wasmtime-rb/pull/149) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump wat from 1.0.58 to 1.0.59 [\#148](https://github.com/bytecodealliance/wasmtime-rb/pull/148) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump magnus from 0.5.0 to 0.5.1 [\#147](https://github.com/bytecodealliance/wasmtime-rb/pull/147) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump benchmark-ips from 2.10.0 to 2.11.0 [\#146](https://github.com/bytecodealliance/wasmtime-rb/pull/146) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump rb\_sys from 0.9.64 to 0.9.65 [\#145](https://github.com/bytecodealliance/wasmtime-rb/pull/145) ([dependabot[bot]](https://github.com/apps/dependabot))
- Fix the build on Ruby \< 3.0 [\#144](https://github.com/bytecodealliance/wasmtime-rb/pull/144) ([jbourassa](https://github.com/jbourassa))
- Bump rb-sys from 0.9.61 to 0.9.64 [\#142](https://github.com/bytecodealliance/wasmtime-rb/pull/142) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump wat from 1.0.57 to 1.0.58 [\#141](https://github.com/bytecodealliance/wasmtime-rb/pull/141) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump rb\_sys from 0.9.61 to 0.9.64 [\#140](https://github.com/bytecodealliance/wasmtime-rb/pull/140) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump standard from 1.22.1 to 1.24.3 [\#139](https://github.com/bytecodealliance/wasmtime-rb/pull/139) ([dependabot[bot]](https://github.com/apps/dependabot))
- Upgrade Magnus to 0.5.0 [\#138](https://github.com/bytecodealliance/wasmtime-rb/pull/138) ([matsadler](https://github.com/matsadler))
- Bump anyhow from 1.0.68 to 1.0.69 [\#137](https://github.com/bytecodealliance/wasmtime-rb/pull/137) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump cap-std from 1.0.4 to 1.0.5 [\#136](https://github.com/bytecodealliance/wasmtime-rb/pull/136) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump wat from 1.0.56 to 1.0.57 [\#134](https://github.com/bytecodealliance/wasmtime-rb/pull/134) ([dependabot[bot]](https://github.com/apps/dependabot))
- Update `rb-sys` to use new `RbSys::ExtensionTask` [\#132](https://github.com/bytecodealliance/wasmtime-rb/pull/132) ([ianks](https://github.com/ianks))
- Tweak codegen flags for optimized, profile-able builds [\#131](https://github.com/bytecodealliance/wasmtime-rb/pull/131) ([ianks](https://github.com/ianks))
- Make ci.yml workflow more trigger happy [\#130](https://github.com/bytecodealliance/wasmtime-rb/pull/130) ([ianks](https://github.com/ianks))
- Ensure source gem builds properly for `cdylib` [\#129](https://github.com/bytecodealliance/wasmtime-rb/pull/129) ([ianks](https://github.com/ianks))
- Document profiling configuration from \#125 [\#127](https://github.com/bytecodealliance/wasmtime-rb/pull/127) ([jbourassa](https://github.com/jbourassa))
- Fix small lint [\#126](https://github.com/bytecodealliance/wasmtime-rb/pull/126) ([ianks](https://github.com/ianks))
- Add support for profiling configuration [\#125](https://github.com/bytecodealliance/wasmtime-rb/pull/125) ([dylanahsmith](https://github.com/dylanahsmith))
- Fix linter [\#124](https://github.com/bytecodealliance/wasmtime-rb/pull/124) ([jbourassa](https://github.com/jbourassa))
- Bump wat from 1.0.55 to 1.0.56 [\#123](https://github.com/bytecodealliance/wasmtime-rb/pull/123) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump tokio from 1.24.2 to 1.25.0 [\#122](https://github.com/bytecodealliance/wasmtime-rb/pull/122) ([dependabot[bot]](https://github.com/apps/dependabot))
- Patches for easier usage in a crate context [\#121](https://github.com/bytecodealliance/wasmtime-rb/pull/121) ([ianks](https://github.com/ianks))
- Skip CI & memcheck when pushing v\* tags [\#120](https://github.com/bytecodealliance/wasmtime-rb/pull/120) ([jbourassa](https://github.com/jbourassa))
- Bump rb\_sys from 0.9.56 to 0.9.58 [\#119](https://github.com/bytecodealliance/wasmtime-rb/pull/119) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump standard from 1.22.0 to 1.22.1 [\#118](https://github.com/bytecodealliance/wasmtime-rb/pull/118) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump rb-sys from 0.9.57 to 0.9.58 [\#117](https://github.com/bytecodealliance/wasmtime-rb/pull/117) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump ruby-lsp from 0.3.5 to 0.3.8 [\#116](https://github.com/bytecodealliance/wasmtime-rb/pull/116) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v5.0.0](https://github.com/bytecodealliance/wasmtime-rb/tree/v5.0.0) (2023-01-23)

[Full Changelog](https://github.com/bytecodealliance/wasmtime-rb/compare/v0.4.1...v5.0.0)

**Merged pull requests:**

- Wasmtime v5 [\#115](https://github.com/bytecodealliance/wasmtime-rb/pull/115) ([jbourassa](https://github.com/jbourassa))
- Add `Memory#slice` for zero-copy data access [\#114](https://github.com/bytecodealliance/wasmtime-rb/pull/114) ([ianks](https://github.com/ianks))
- Add `Memory#read_utf8` [\#113](https://github.com/bytecodealliance/wasmtime-rb/pull/113) ([ianks](https://github.com/ianks))
- Implement GC compaction for store data [\#112](https://github.com/bytecodealliance/wasmtime-rb/pull/112) ([ianks](https://github.com/ianks))
- Bump rb-sys from 0.9.54 to 0.9.56 [\#111](https://github.com/bytecodealliance/wasmtime-rb/pull/111) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump rb\_sys from 0.9.54 to 0.9.56 [\#110](https://github.com/bytecodealliance/wasmtime-rb/pull/110) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump standard from 1.21.1 to 1.22.0 [\#109](https://github.com/bytecodealliance/wasmtime-rb/pull/109) ([dependabot[bot]](https://github.com/apps/dependabot))
- Version oxidize-rb actions [\#108](https://github.com/bytecodealliance/wasmtime-rb/pull/108) ([ianks](https://github.com/ianks))
- Bump tokio from 1.23.1 to 1.24.1 [\#107](https://github.com/bytecodealliance/wasmtime-rb/pull/107) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump standard from 1.20.0 to 1.21.1 [\#106](https://github.com/bytecodealliance/wasmtime-rb/pull/106) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump rb-sys to 0.9.54 for Ruby 3.2 [\#105](https://github.com/bytecodealliance/wasmtime-rb/pull/105) ([jbourassa](https://github.com/jbourassa))
- Bump tokio from 1.23.0 to 1.23.1 [\#104](https://github.com/bytecodealliance/wasmtime-rb/pull/104) ([dependabot[bot]](https://github.com/apps/dependabot))

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
