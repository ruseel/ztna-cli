//! Read-only Accessibility permission check; no AppleScript or subprocesses.

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    // macOS Boolean is an unsigned byte, not an Objective-C BOOL.
    fn AXIsProcessTrusted() -> u8;
}

pub fn check() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        // SAFETY: AXIsProcessTrusted takes no arguments and has no ownership or
        // thread requirements. Unlike the WithOptions API it never prompts.
        report(unsafe { AXIsProcessTrusted() } != 0)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("ztna-cli doctor only supports macOS".into())
    }
}

#[cfg(any(target_os = "macos", test))]
fn report(trusted: bool) -> Result<String, String> {
    if trusted {
        Ok("Accessibility: enabled for this process.\nAutomation permission for System Events is separate; run `ztna-cli status` to verify end-to-end access.".into())
    } else {
        Err("Accessibility: NOT enabled for this process.\nOpen System Settings → Privacy & Security → Accessibility.\nEnable your terminal app or add/enable the installed ztna-cli executable, then restart the terminal/CLI and run `ztna-cli doctor` again.\nAutomation permission for System Events is separate and is not checked here.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_is_success() {
        let text = report(true).unwrap();
        assert!(text.contains("Accessibility: enabled"));
        assert!(text.contains("Automation permission"));
    }

    #[test]
    fn disabled_is_failure_with_remediation() {
        let text = report(false).unwrap_err();
        assert!(text.contains("Accessibility: NOT enabled"));
        assert!(text.contains("System Settings"));
    }
}
