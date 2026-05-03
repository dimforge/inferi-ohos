# inferi-ohos

HarmonyOS demo app that runs LLM inference on-device using
[inferi](https://github.com/dimforge/inferi). The Rust core is compiled as a
`cdylib` and exposed to ArkTS via `napi-ohos`.

## Prerequisites

- **DevEco Studio** with the OpenHarmony SDK installed. The build scripts assume
  the macOS install path:
  `/Applications/DevEco-Studio.app/Contents/sdk/default/openharmony`.
  Adjust `build.sh` (top-level and `rust/build.sh`) if yours differs.
- **Rust toolchain** with the OpenHarmony targets:
  ```sh
  rustup target add aarch64-unknown-linux-ohos x86_64-unknown-linux-ohos
  ```
- Optional: [`ohrs`](https://crates.io/crates/ohrs) — if present, `rust/build.sh`
  uses it; otherwise it falls back to `cargo` with a linker wrapper.
- Local checkouts of `inferi`, `khal`, and `vortx` at the paths referenced by
  `rust/Cargo.toml` (`/Users/sebcrozet/work/...`). Update those paths to match
  your machine.
- A GGUF model placed at `entry/src/main/resources/rawfile/model.gguf` (it is
  bundled into the HAP and extracted to the app's files dir on first launch).

## Build

The top-level `build.sh` runs the full pipeline (Rust → copy `libc++_shared.so`
→ assemble HAP):

```sh
./build.sh
```

This produces a HAP under `entry/build/`. To rebuild only the Rust side:

```sh
export OHOS_NDK_HOME=/Applications/DevEco-Studio.app/Contents/sdk/default/openharmony
cd rust && ./build.sh manual   # or just `./build.sh` to use ohrs
```

The Rust step writes `libinferi_ohos.so` into `entry/libs/arm64-v8a/` and
`entry/libs/x86_64/`.

## Run

1. Open the project in DevEco Studio.
2. Connect a HarmonyOS device or start an emulator.
3. Hit **Run**. On launch, tap **Load Model**, wait for status to reach
   `Ready — ...`, enter a prompt, then tap **Generate**.
