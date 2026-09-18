//! In-process AppleScript execution. No shell or child process is used.
use std::time::Duration;

#[cfg(not(target_os = "macos"))]
pub fn run(_: &str, _: Duration) -> Result<String, String> {
    Err("ztna-cli only supports macOS".into())
}

#[cfg(target_os = "macos")]
pub fn run(operation: &str, timeout: Duration) -> Result<String, String> {
    use std::{process, sync::mpsc, thread};

    let source = source(operation)?;
    // NSAppleScript runs on the main thread. A watchdog exits the CLI if native
    // execution blocks; cancelling AppleScript on another thread is not safe.
    let (done_tx, done_rx) = mpsc::channel();
    let operation = operation.to_owned();
    let watchdog = thread::spawn(move || {
        if matches!(
            done_rx.recv_timeout(timeout),
            Err(mpsc::RecvTimeoutError::Timeout)
        ) {
            eprintln!(
                "ztna-cli: {operation} timed out after {}s. The VPN operation may still be in progress; check `ztna-cli status`. Complete any authentication prompts, or retry with --timeout.",
                timeout.as_secs()
            );
            process::exit(1);
        }
    });
    let result = execute(&source);
    let _ = done_tx.send(());
    let _ = watchdog.join();
    result
}

#[cfg(any(target_os = "macos", test))]
fn source(operation: &str) -> Result<String, String> {
    // Only fixed command literals may be embedded in AppleScript source.
    if !matches!(operation, "connect" | "disconnect" | "status") {
        return Err(format!("Unsupported operation: {operation}"));
    }
    Ok(format!(
        "{}\nperformOperation(\"{operation}\")\n",
        include_str!("globalprotect.applescript")
    ))
}

#[cfg(target_os = "macos")]
fn execute(source: &str) -> Result<String, String> {
    use objc2::{
        msg_send,
        rc::{autoreleasepool, Retained},
        runtime::AnyObject,
        AnyThread,
    };
    use objc2_foundation::{NSAppleEventDescriptor, NSAppleScript, NSDictionary, NSString};

    autoreleasepool(|_| {
        let source = NSString::from_str(source);
        let script = NSAppleScript::initWithSource(NSAppleScript::alloc(), &source)
            .ok_or("Could not initialize NSAppleScript")?;
        let mut error: Option<Retained<NSDictionary<NSString, AnyObject>>> = None;
        // SAFETY: executed on the CLI's main thread with a live script and an
        // out-parameter of the documented NSDictionary<NSString, id> type.
        // Apple's API may return nil on failure; the generated binding declares
        // a non-null return, so use msg_send! with the correct nullable type.
        let result: Option<Retained<NSAppleEventDescriptor>> =
            unsafe { msg_send![&*script, executeAndReturnError: &mut error] };
        if let Some(error) = error {
            return Err(format!(
                "GlobalProtect automation failed: {error:?}\nCheck System Settings → Privacy & Security → Accessibility and Automation permissions for your terminal or ztna-cli."
            ));
        }
        let output = result
            .and_then(|value| value.stringValue())
            .ok_or("GlobalProtect returned no text status")?
            .to_string();
        if output.trim().is_empty() {
            return Err("GlobalProtect returned an empty status".into());
        }
        Ok(output.trim().to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_invokes_each_supported_operation() {
        for operation in ["connect", "disconnect", "status"] {
            assert!(source(operation)
                .unwrap()
                .ends_with(&format!("performOperation(\"{operation}\")\n")));
        }
    }

    #[test]
    fn rejects_source_injection() {
        assert!(source("status\")\ndo shell script \"anything").is_err());
    }
}
