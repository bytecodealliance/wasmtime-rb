// Small script used to calculate the native gem build matrix.
//
// Keep this in one place so CI and release workflows build the same set of
// platform gems.

const linuxX64 = "ubuntu-22.04";
const linuxArm64 = "ubuntu-24.04-arm";
const windows = "windows-2025";
const macos = "macos-15";

const builds = [
  {
    "ruby-platform": "x86_64-linux",
    "rust-target": "x86_64-unknown-linux-gnu",
    os: linuxX64,
    "smoke-test": true,
  },
  {
    "ruby-platform": "aarch64-linux",
    "rust-target": "aarch64-unknown-linux-gnu",
    os: linuxArm64,
    "smoke-test": true,
  },
  {
    "ruby-platform": "x86_64-linux-musl",
    "rust-target": "x86_64-unknown-linux-musl",
    os: linuxX64,
    "apt-packages": "musl-tools",
    cc: "musl-gcc",
    rustflags: "-Ctarget-feature=-crt-static",
  },
  {
    "ruby-platform": "aarch64-linux-musl",
    "rust-target": "aarch64-unknown-linux-musl",
    os: linuxArm64,
    "apt-packages": "musl-tools",
    cc: "musl-gcc",
    rustflags: "-Ctarget-feature=-crt-static",
  },
  {
    "ruby-platform": "x86_64-darwin",
    "rust-target": "x86_64-apple-darwin",
    os: macos,
  },
  {
    "ruby-platform": "arm64-darwin",
    "rust-target": "aarch64-apple-darwin",
    os: macos,
  },
  {
    "ruby-platform": "x64-mingw-ucrt",
    "rust-target": "x86_64-pc-windows-gnu",
    os: windows,
    "smoke-test": true,
  },
  {
    "ruby-platform": "aarch64-mingw-ucrt",
    "rust-target": "aarch64-pc-windows-gnullvm",
    os: windows,
    "llvm-mingw": true,
    cc: "aarch64-w64-mingw32-clang",
    cxx: "aarch64-w64-mingw32-clang++",
    ar: "llvm-ar",
  },
];

console.log(JSON.stringify(builds));
