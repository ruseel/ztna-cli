---
name: ztna-cli
description: Install and use ruseel/ztna-cli to check, connect, or disconnect GlobalProtect VPN on macOS. Use for ztna-cli setup via mise, command-not-found troubleshooting, Accessibility permission checks, or requests to control GlobalProtect/ ZTNA connectivity. Does not handle Azure authentication.
compatibility: macOS Apple Silicon or Intel, mise, and the GlobalProtect app. UI control requires Accessibility and Automation permissions.
---

# ztna-cli

A macOS CLI that controls GlobalProtect through its menu-bar UI. The AppleScript
is embedded in the binary; no external scripts, sudo, or credentials are needed.

## Safety

- Only connect or disconnect when requested by the user. Status and doctor checks
  do not intentionally change the VPN connection.
- For a requested connect/disconnect test, record the initial status and restore
  that state afterward. Do not disconnect an initially connected VPN merely to
  finish a test without making that consequence clear to the user.
- Never change routes, certificates, VPN settings, or stored credentials to fix
  a CLI error. Do not bypass authentication or approve permission prompts for
  the user.
- Do not run legacy AppleScripts as a fallback: use the installed CLI.

## Installation and invocation

Use the GitHub release through mise (macOS ARM64 and Intel are supported):

```sh
mise use -g github:ruseel/ztna-cli@latest
mise exec -- ztna-cli --version
```

`mise use -g` changes the user's global tool selection. Run it when installation
or global setup is requested, not on every invocation. For existing installs,
prefer `mise exec -- ztna-cli ...`; it works without interactive shell activation.
To explicitly select this backend regardless of project configuration:

```sh
mise exec github:ruseel/ztna-cli@0.1.0 -- ztna-cli --version
```

The version above is an example pin, not an instruction to downgrade. Inspect
available versions with `mise ls-remote github:ruseel/ztna-cli` when needed.

`mise which ztna-cli` reports the selected executable. The default installation
is under `~/.local/share/mise/installs/`, not `~/.local/bin`. Do not hardcode an
absolute version directory, copy the binary, or create a wrapper unnecessarily.

### zsh: command not found

Installation does not by itself add the executable directory to the current
shell's PATH. If the user wants to invoke `ztna-cli` directly, add this line once
to `~/.zshrc` (check for an existing activation first):

```zsh
eval "$(mise activate zsh)"
```

Then have the user run in their zsh terminal:

```zsh
source ~/.zshrc
ztna-cli --version
```

Do not execute these zsh snippets in Nushell. Agent commands can continue using
`mise exec` without altering shell configuration. In Nushell, invoke external
commands as `^mise ...` and use `| complete` to capture stdout, stderr, and
`exit_code`, especially for expected diagnostic failures.

## Check before changing state

```sh
mise exec -- ztna-cli doctor
mise exec -- ztna-cli status
```

- `doctor` checks this process's Accessibility trust. Exit 0 means enabled;
  exit 1 includes remediation. It neither prompts nor checks Automation access.
- If Accessibility is missing, ask the user to allow the terminal/execution
  context in System Settings → Privacy & Security → Accessibility.
- `status` verifies end-to-end UI access. Automation access to System Events is
  separate; ask the user to approve it when prompted. Run diagnostics in the
  same terminal/context as the eventual connect/disconnect command.
- Status prints GlobalProtect's menu-bar help text, e.g. `Connected`,
  `Not Connected`, transitional text, or `Not running`. Exit 0 does **not** mean
  connected. Never use substring matching for `Connected`: `Not Connected`
  contains that word too.
- If GlobalProtect is not running, ask the user to launch it before connecting.

## Connect or disconnect

Run only the requested operation, then independently check status:

```sh
mise exec -- ztna-cli connect
mise exec -- ztna-cli status
```

```sh
mise exec -- ztna-cli disconnect
mise exec -- ztna-cli status
```

Commands wait for the target state and are safe to repeat when already in that
state. Authentication may require the user to interact with a browser or the
GlobalProtect UI. Do not request or capture their password/MFA code.

Default timeouts: connect/disconnect 120 seconds; status 10 seconds. To allow
more time for interactive authentication:

```sh
mise exec -- ztna-cli connect --timeout 300
```

`--timeout` accepts 1–3600 seconds for connect/disconnect/status; doctor does not
accept it. Set the agent tool execution timeout slightly above the CLI timeout.
A CLI timeout ends the CLI, **not** an already-started VPN transition. After a
timeout, check status before retrying or attempting restoration.

## Reporting and troubleshooting

- Exit codes: 0 success (including disconnected status), 1 automation error or
  timeout, 2 invalid arguments.
- Report the executable version, operation result, and separately verified final
  status. Do not claim network reachability based solely on a UI status check.
- The CLI resolves its button and state labels from GlobalProtect's installed
  localization resources, including Korean. If the menu-bar structure has
  changed, report that limitation rather than repeatedly attempting UI control.
- This skill does not authenticate Azure CLI or configure tenant certificates;
  handle those separately if requested.
