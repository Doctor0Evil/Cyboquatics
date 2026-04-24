## Ecosafety Toolchain Spec: Rust-on‑mnt/oss Only

**Spec ID:** `ToolchainPolicyRustOnOss2026v1`  
**Status:** REQUIRED for all lanes (RESEARCH, EXPERIMENTAL, PILOT, PROD) in this workspace. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/ae4448b9-cf07-4dff-a363-481213e834aa/this-document-provides-exactly-3WKgRVZ9Tv.H44433auPkQ.md)

### 1. Normative requirement

1. Rust toolchains **MUST NOT** be installed on the root filesystem of the host or container running this workspace. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/2bbd9d7b-5f6e-41e4-b374-308a1c94fb4f/100-answers-for-research-conti-wwGKbDBjT2uH0JS.NVLzeQ.md)
2. All Rust tooling (**rustc**, **cargo**, **rustfmt**, **clippy**, rustup metadata) **MUST** reside under the governed VFS root `mnt/oss`, using: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/ae4448b9-cf07-4dff-a363-481213e834aa/this-document-provides-exactly-3WKgRVZ9Tv.H44433auPkQ.md)
   - `RUSTUP_HOME = /mnt/oss/rustup`  
   - `CARGO_HOME = /mnt/oss/cargo`  
3. All builds, checks, and tests in this repository **MUST** run with `CARGO_HOME/bin` on `PATH` and **MUST NOT** invoke system-level or distro-installed Rust binaries. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/2bbd9d7b-5f6e-41e4-b374-308a1c94fb4f/100-answers-for-research-conti-wwGKbDBjT2uH0JS.NVLzeQ.md)

If any of these conditions are violated, ecosafety CI **MUST** fail with `kerdeployable = false` for this repo’s toolchain context. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/ae4448b9-cf07-4dff-a363-481213e834aa/this-document-provides-exactly-3WKgRVZ9Tv.H44433auPkQ.md)

***

### 2. Environment contract

A coding session or CI job is considered **valid** only if, before running any `cargo` or `rustc` command, it executes the shared environment script on `mnt/oss`: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/2bbd9d7b-5f6e-41e4-b374-308a1c94fb4f/100-answers-for-research-conti-wwGKbDBjT2uH0JS.NVLzeQ.md)

```bash
# Canonical toolchain environment
export RUSTUP_HOME="/mnt/oss/rustup"
export CARGO_HOME="/mnt/oss/cargo"
export PATH="${CARGO_HOME}/bin:${PATH}"
```

Local wrappers (for IDEs, scripts, or agents) **MUST** source this script or inline equivalent exports and **MUST** check that `cargo` is resolved from `/mnt/oss/cargo/bin`, not `/usr` or similar paths. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/ae4448b9-cf07-4dff-a363-481213e834aa/this-document-provides-exactly-3WKgRVZ9Tv.H44433auPkQ.md)

If `cargo` is missing under `/mnt/oss/cargo/bin`, tools **MUST** print a clear diagnostic:

> “Rust toolchain not found under /mnt/oss. Ask the maintainer to (re)install it on the VFS. Do **not** install Rust into the root filesystem.” [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/2bbd9d7b-5f6e-41e4-b374-308a1c94fb4f/100-answers-for-research-conti-wwGKbDBjT2uH0JS.NVLzeQ.md)

and abort, rather than attempting any install themselves. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/ae4448b9-cf07-4dff-a363-481213e834aa/this-document-provides-exactly-3WKgRVZ9Tv.H44433auPkQ.md)

***

### 3. Prohibited behaviors

The following actions are **forbidden** for coders and agents in this workspace:

1. Running any of:  
   - `curl https://sh.rustup.rs | sh`  
   - `rustup-init`  
   - `apt-get install rustc cargo`  
   unless the maintainer explicitly runs them with `RUSTUP_HOME=/mnt/oss/rustup`, `CARGO_HOME=/mnt/oss/cargo` and outside of CI/agent flows. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/2bbd9d7b-5f6e-41e4-b374-308a1c94fb4f/100-answers-for-research-conti-wwGKbDBjT2uH0JS.NVLzeQ.md)

2. Modifying `/usr`, `/usr/local`, or any non‑`mnt/oss` path to host Rust binaries or rustup metadata for this project. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/ae4448b9-cf07-4dff-a363-481213e834aa/this-document-provides-exactly-3WKgRVZ9Tv.H44433auPkQ.md)

3. Adding alternative Rust locations to `PATH` ahead of `/mnt/oss/cargo/bin` in workspace config files, CI workflows, or agent presets. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/2bbd9d7b-5f6e-41e4-b374-308a1c94fb4f/100-answers-for-research-conti-wwGKbDBjT2uH0JS.NVLzeQ.md)

Any script that tries to install Rust outside `mnt/oss` must be treated as a **schema violation** for the toolchain environment shard and blocked with high `rcalib` and `R` for the run. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/ae4448b9-cf07-4dff-a363-481213e834aa/this-document-provides-exactly-3WKgRVZ9Tv.H44433auPkQ.md)

***

### 4. Workspace wiring rules

Within this repository:

1. All `cargo` invocations in documentation, scripts, and CI examples **MUST** be wrapped in a project-local script that enforces the `mnt/oss` environment, for example: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/2bbd9d7b-5f6e-41e4-b374-308a1c94fb4f/100-answers-for-research-conti-wwGKbDBjT2uH0JS.NVLzeQ.md)

```bash
./workspace/.tools/env-ecosafety.sh cargo check -p ecosafety-core
```

2. `.envrc`, CI YAML, and editor tasks **MUST** rely on `CARGO_HOME` and **MUST NOT** introduce a conflicting `RUSTROOT` or similar variable pointing elsewhere. Legacy `RUSTROOT` entries **MUST** either be removed or set to the same directory as `CARGO_HOME` with a “deprecated – do not use” comment. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/ae4448b9-cf07-4dff-a363-481213e834aa/this-document-provides-exactly-3WKgRVZ9Tv.H44433auPkQ.md)

3. Any new tooling added to this workspace that needs Rust (e.g., `cargo-ecosafety`, `cargo-ecosafety-watch`) **MUST** assume the canonical `mnt/oss` toolchain and **MUST NOT** attempt self-installation. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/2bbd9d7b-5f6e-41e4-b374-308a1c94fb4f/100-answers-for-research-conti-wwGKbDBjT2uH0JS.NVLzeQ.md)
