# ztna-cli

Rust CLI for GlobalProtect on macOS, based on the AppleScript
implementations in `~/bin/globalprotect-connect.applescript` and
`~/bin/globalprotect-disconnect.applescript`.

## Install

```sh
cargo install --path .
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

- macOS, Rust/Cargo to build, and the GlobalProtect app running for connect/disconnect.
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
