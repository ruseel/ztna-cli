# ztna-cli

Rust CLI for GlobalProtect on macOS, based on the AppleScript
implementations in `~/bin/globalprotect-connect.applescript` and
`~/bin/globalprotect-disconnect.applescript`.

## Install

### mise (recommended)

Install the latest release from `ruseel/ztna-cli`:

```sh
mise use -g github:ruseel/ztna-cli@latest
mise exec -- ztna-cli --version
ztna-cli doctor
```

Requires [mise](https://mise.jdx.dev/getting-started.html) activated in your shell.
Omit `-g` to record the tool in the current project's `mise.toml` instead.
Prebuilt binaries are available for macOS Apple Silicon and Intel; Rust is not
needed. Linux and Windows are not supported.

To list releases or pin a version:

```sh
mise ls-remote github:ruseel/ztna-cli
mise use -g github:ruseel/ztna-cli@0.1.0
```

The [mise GitHub backend](https://mise.jdx.dev/dev-tools/backends/github.html)
automatically selects the archive for your architecture. No plugin or custom
asset pattern is needed. It requires uploaded release assets, not just a Git tag
or GitHub's automatically generated source archives.

### Build from source

```sh
cargo install --locked --path .
```

This installs `~/.cargo/bin/ztna-cli`; ensure `~/.cargo/bin` is on your PATH.
Alternatively, run locally with `cargo run -- status`.

## Usage

```sh
ztna-cli doctor
ztna-cli status
ztna-cli connect
ztna-cli disconnect
ztna-cli connect --timeout 300
```

Connect/disconnect wait for the target status and are safe to repeat when already
in that state. Default timeout: 120 seconds; status: 10 seconds. Override with
`--timeout SECONDS` (1–3600). Authentication may require browser/UI interaction.
A timeout terminates the CLI (via a watchdog), **not** an already-started VPN
transition. Native AppleScript executes on the main thread.

Status prints GlobalProtect's menu-bar help text verbatim (e.g. `Connected`,
`Not Connected`, or a transitional state), or `Not running` if the app is closed.
Exit codes: 0 on success, 1 on automation errors/timeouts, 2 on invalid arguments.
A disconnected or not-running status still exits 0; this is not a connectivity probe.

## Doctor

`ztna-cli doctor` checks Accessibility trust for the current process using the
native macOS `AXIsProcessTrusted` API. It does not launch subprocesses, prompt
for permissions, or change your VPN connection. Exit code is 0 if enabled, 1 if
missing (with setup instructions). Run it from the same terminal/context you
use for connect/disconnect, since permissions depend on the execution context.

Automation permission for System Events is separate and is not checked by
`doctor`; use `ztna-cli status` to verify end-to-end UI access.

## Requirements and limitations

- macOS and the GlobalProtect app running for connect/disconnect. Rust/Cargo is
  required only when building from source.
- Allow your terminal under System Settings → Privacy & Security → Accessibility,
  and allow Automation access to System Events when prompted.
- Uses the same English UI labels and menu-bar accessibility structure as the
  original scripts. Other languages or GlobalProtect UI changes may need updates
  to `src/globalprotect.applescript`.
- No sudo, credentials, route changes, or external scripts are required at runtime.
  The AppleScript is embedded in the binary and executed in-process using macOS
  Foundation's `NSAppleScript` through `objc2` bindings. No shell or `osascript`
  subprocess is launched. This still uses AppleScript/System Events UI automation,
  not a private GlobalProtect networking API.
- Disconnect dismisses the GlobalProtect menu-bar panel by toggling its icon
  when the panel is open, including when already disconnected. It never sends
  a global Cmd+W, avoiding accidentally closing another app's window.

## Development

```sh
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

Unit tests cover CLI parsing. Live connect/disconnect testing changes your VPN
connection and should be performed manually.

## Publishing releases (maintainers)

The repository must be available at `ruseel/ztna-cli` for the installation commands
above. For installation without GitHub authentication, it must be public. Private
repositories require each user to have access and configure a
[mise GitHub token](https://mise.jdx.dev/dev-tools/github-tokens.html).

1. Set the version in `Cargo.toml` and update `Cargo.lock` with `cargo check`.
2. Commit and push the changes, including `.github/workflows/release.yml`.
3. Push a matching version tag (the workflow rejects a Cargo version mismatch):

   ```sh
   git tag v0.1.0
   git push origin v0.1.0
   ```

The release workflow tests and builds on Apple Silicon and Intel macOS runners,
then publishes a GitHub Release only after both builds succeed. Assets:

- `ztna-cli-v0.1.0-aarch64-apple-darwin.tar.gz`
- `ztna-cli-v0.1.0-x86_64-apple-darwin.tar.gz`
- `SHA256SUMS`

Each archive contains the executable `ztna-cli` at its root, with the executable
permission preserved. Only `--help` and `--version` are smoke-tested in CI; no VPN
connection or UI automation is attempted.

After publishing, verify the installation on each supported architecture:

```sh
mise use -g github:ruseel/ztna-cli@0.1.0
mise exec -- ztna-cli --version
```

If publishing fails after creating the draft release, delete that draft (keep the
tag) before rerunning the workflow.
