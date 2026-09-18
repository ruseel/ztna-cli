use std::{env, process, time::Duration};

mod doctor;
mod native;

const HELP: &str = "ztna-cli — GlobalProtect control for macOS

Usage: ztna-cli <connect|disconnect|status> [--timeout SECONDS]
       ztna-cli doctor

  connect       Connect, waiting until connected (default timeout: 120s)
  disconnect    Disconnect, waiting until disconnected (default timeout: 120s)
  status        Print current menu-bar status (default timeout: 10s)
  doctor        Check Accessibility permission without prompting

  --timeout N   Overall timeout in seconds (1–3600)
  -h, --help    Show help
  -V, --version Show version

Requires GlobalProtect and macOS Accessibility/Automation permissions.
Authentication may require interaction with your browser or GlobalProtect.
Exit codes: 0 success (including disconnected status), 1 failure, 2 usage error.
";

#[derive(Debug, PartialEq)]
enum Action {
    Help,
    Version,
    Doctor,
    Run {
        operation: String,
        timeout: Duration,
    },
}

fn parse(args: impl IntoIterator<Item = String>) -> Result<Action, String> {
    let mut args = args.into_iter();
    let Some(operation) = args.next() else {
        return Ok(Action::Help);
    };
    if matches!(operation.as_str(), "-h" | "--help" | "-V" | "--version") {
        if args.next().is_some() {
            return Err("Unexpected extra arguments".into());
        }
        return Ok(if matches!(operation.as_str(), "-h" | "--help") {
            Action::Help
        } else {
            Action::Version
        });
    }
    if operation == "doctor" {
        return match (args.next().as_deref(), args.next()) {
            (None, None) => Ok(Action::Doctor),
            (Some("-h" | "--help"), None) => Ok(Action::Help),
            _ => Err("doctor does not accept arguments; use `ztna-cli doctor`".into()),
        };
    }
    if !matches!(operation.as_str(), "connect" | "disconnect" | "status") {
        return Err(format!("Unknown command: {operation}"));
    }
    let mut seconds = if operation == "status" { 10 } else { 120 };
    let mut timeout_seen = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Action::Help),
            "--timeout" if !timeout_seen => {
                timeout_seen = true;
                seconds = args
                    .next()
                    .and_then(|value| value.parse::<u64>().ok())
                    .filter(|value| (1..=3600).contains(value))
                    .ok_or("--timeout requires an integer from 1 to 3600")?;
            }
            _ => return Err(format!("Unexpected argument: {arg}")),
        }
    }
    Ok(Action::Run {
        operation,
        timeout: Duration::from_secs(seconds),
    })
}

fn main() {
    match parse(env::args().skip(1)) {
        Ok(Action::Help) => print!("{HELP}"),
        Ok(Action::Version) => println!("ztna-cli {}", env!("CARGO_PKG_VERSION")),
        Ok(Action::Doctor) => match doctor::check() {
            Ok(report) => println!("{report}"),
            Err(report) => {
                eprintln!("{report}");
                process::exit(1);
            }
        },
        Ok(Action::Run { operation, timeout }) => match native::run(&operation, timeout) {
            Ok(status) => println!("{status}"),
            Err(error) => {
                eprintln!("ztna-cli: {error}");
                process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("ztna-cli: {error}\nRun `ztna-cli --help` for usage.");
            process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cli(args: &[&str]) -> Result<Action, String> {
        parse(args.iter().map(|arg| arg.to_string()))
    }

    #[test]
    fn help_and_version() {
        assert_eq!(cli(&[]).unwrap(), Action::Help);
        assert_eq!(cli(&["--help"]).unwrap(), Action::Help);
        assert_eq!(cli(&["connect", "--help"]).unwrap(), Action::Help);
        assert_eq!(cli(&["--version"]).unwrap(), Action::Version);
    }

    #[test]
    fn doctor_command() {
        assert_eq!(cli(&["doctor"]).unwrap(), Action::Doctor);
        assert_eq!(cli(&["doctor", "--help"]).unwrap(), Action::Help);
        assert!(cli(&["doctor", "--timeout", "10"]).is_err());
        assert!(cli(&["doctor", "extra"]).is_err());
        assert!(cli(&["doctor", "--help", "extra"]).is_err());
    }

    #[test]
    fn commands_and_defaults() {
        for (operation, seconds) in [("connect", 120), ("disconnect", 120), ("status", 10)] {
            assert_eq!(
                cli(&[operation]).unwrap(),
                Action::Run {
                    operation: operation.into(),
                    timeout: Duration::from_secs(seconds),
                }
            );
        }
    }

    #[test]
    fn custom_timeout() {
        assert_eq!(
            cli(&["connect", "--timeout", "300"]).unwrap(),
            Action::Run {
                operation: "connect".into(),
                timeout: Duration::from_secs(300),
            }
        );
    }

    #[test]
    fn rejects_invalid_arguments() {
        for args in [
            vec!["invalid"],
            vec!["status", "extra"],
            vec!["connect", "--timeout"],
            vec!["connect", "--timeout", "0"],
            vec!["connect", "--timeout", "-1"],
            vec!["connect", "--timeout", "3601"],
            vec!["connect", "--timeout", "abc"],
            vec!["connect", "--timeout", "1", "--timeout", "2"],
            vec!["--help", "extra"],
        ] {
            assert!(cli(&args).is_err(), "{args:?}");
        }
    }
}
