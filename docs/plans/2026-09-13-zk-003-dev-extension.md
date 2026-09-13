# ZK-003 — Connect the probe to Zed

Authorized scope: build the disposable dev extension and prepare/perform the real
editor connection check for the next backlog task.

1. Install the WASM target and pin the extension API dependency.
2. Register a probe-only `.kata` language, reuse the Rust grammar for the Rust-shaped
   fixture, and launch the compiled probe using Node discovered in the worktree.
3. Add a launcher that creates unique captures under the probe directory, independent
   of Zed's working directory, and add exact installation/smoke-test instructions.
4. Build the extension, run formatting/clippy and probe checks, then open the repo
   in Zed. Record only observed results; leave ZK-003 unchecked if UI installation,
   editing, or visible diagnostic confirmation requires Dan's input.

The extension is disposable spike code. Production toolchain decisions remain deferred.
