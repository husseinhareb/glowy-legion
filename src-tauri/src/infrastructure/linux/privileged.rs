use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// We run privileged commands through `sudo -S`, reading the password from
/// stdin. The password is supplied by the user through the app's authentication
/// dialog, passed to this function once, written to sudo's stdin, and never
/// stored, logged, or placed on the process command line.
pub const SUDO_BIN: &str = "sudo";

#[derive(Debug)]
pub enum PrivilegedError {
    /// `sudo` is not installed / not on PATH.
    SudoMissing,
    /// The privileged process could not be spawned at all.
    Spawn(String),
}

impl PrivilegedError {
    pub fn user_message(&self) -> String {
        match self {
            PrivilegedError::SudoMissing => {
                "`sudo` is not available on this system. Use the manual terminal commands instead."
                    .to_string()
            }
            PrivilegedError::Spawn(detail) => {
                format!("Could not start the privileged command: {detail}.")
            }
        }
    }
}

/// Captured output of a privileged process. Never contains the password.
#[derive(Debug, Clone)]
pub struct PrivilegedOutcome {
    pub success: bool,
    /// Exit code, retained for diagnostics even when not currently read.
    #[allow(dead_code)]
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub incorrect_password: bool,
}

/// Whether `sudo` can be found on PATH. Pure lookup with no side effects.
pub fn sudo_available() -> bool {
    find_in_path(SUDO_BIN).is_some()
}

fn find_in_path(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(bin))
        .find(|candidate| candidate.is_file())
}

/// Whether a sudo failure looks like a bad password rather than a command error.
pub fn looks_like_incorrect_password(stderr: &str) -> bool {
    let lowered = stderr.to_ascii_lowercase();
    lowered.contains("incorrect password")
        || lowered.contains("authentication failure")
        || lowered.contains("sorry, try again")
        || lowered.contains("no password was provided")
        || lowered.contains("a password is required")
}

/// Run a privileged command via `sudo -S`, supplying `password` on stdin.
///
/// The command and its arguments are passed directly to the process (no shell),
/// so there is no string interpolation and no shell-injection surface. The
/// password is written to stdin only; it never appears in the argument vector,
/// the process list, stdout, or stderr.
pub fn run_with_sudo(password: &str, argv: &[&str]) -> Result<PrivilegedOutcome, PrivilegedError> {
    if !sudo_available() {
        return Err(PrivilegedError::SudoMissing);
    }

    // -S: read the password from stdin. -k: ignore any cached credentials so the
    // supplied password is always what is checked. -p "": suppress sudo's own
    // prompt text so nothing is echoed.
    let mut child = Command::new(SUDO_BIN)
        .arg("-S")
        .arg("-k")
        .arg("-p")
        .arg("")
        .args(argv)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| PrivilegedError::Spawn(error.to_string()))?;

    if let Some(mut stdin) = child.stdin.take() {
        // Best-effort write; if sudo exits early (e.g. cached creds) the pipe may
        // already be closed, which is fine. stdin is dropped here, closing it.
        let _ = stdin.write_all(format!("{password}\n").as_bytes());
    }

    let output = child
        .wait_with_output()
        .map_err(|error| PrivilegedError::Spawn(error.to_string()))?;

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let incorrect_password = !output.status.success() && looks_like_incorrect_password(&stderr);

    Ok(PrivilegedOutcome {
        success: output.status.success(),
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        stderr,
        incorrect_password,
    })
}

/// Argument vector for installing a generated rule file to a fixed destination.
/// `install -m 0644 <source> <dest>` avoids any shell quoting.
pub fn install_rule_argv(source: &Path, dest: &str) -> Vec<String> {
    vec![
        "install".to_string(),
        "-m".to_string(),
        "0644".to_string(),
        source.display().to_string(),
        dest.to_string(),
    ]
}

/// Argument vector for removing the LegionGlow-managed rule file.
pub fn remove_rule_argv(path: &str) -> Vec<String> {
    vec!["rm".to_string(), "-f".to_string(), path.to_string()]
}

/// `udevadm control --reload-rules`.
pub fn reload_rules_argv() -> Vec<String> {
    vec![
        "udevadm".to_string(),
        "control".to_string(),
        "--reload-rules".to_string(),
    ]
}

/// `udevadm trigger` (generic change event for all devices).
pub fn trigger_argv() -> Vec<String> {
    vec!["udevadm".to_string(), "trigger".to_string()]
}

/// Re-emit an **add** uevent for exactly the matching USB device.
///
/// This is what makes a freshly installed `uaccess` rule take effect on an
/// already-connected, non-removable device (e.g. an internal laptop keyboard)
/// without a replug: systemd-logind reapplies the ACL on the add event. The
/// match is narrowed to the one vendor/product so no other devices are touched.
pub fn trigger_add_argv(vendor_id: u16, product_id: u16) -> Vec<String> {
    vec![
        "udevadm".to_string(),
        "trigger".to_string(),
        "--action=add".to_string(),
        "--subsystem-match=usb".to_string(),
        format!("--attr-match=idVendor={vendor_id:04x}"),
        format!("--attr-match=idProduct={product_id:04x}"),
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        install_rule_argv, looks_like_incorrect_password, reload_rules_argv, remove_rule_argv,
        trigger_add_argv, trigger_argv, PrivilegedError,
    };
    use std::path::Path;

    #[test]
    fn targeted_add_trigger_matches_only_the_device() {
        let argv = trigger_add_argv(0x048d, 0xc693);

        assert_eq!(
            argv,
            vec![
                "udevadm",
                "trigger",
                "--action=add",
                "--subsystem-match=usb",
                "--attr-match=idVendor=048d",
                "--attr-match=idProduct=c693",
            ]
        );
    }

    #[test]
    fn install_argv_uses_fixed_mode_and_install_binary() {
        let argv = install_rule_argv(
            Path::new("/tmp/legionglow-rule.rules"),
            "/etc/udev/rules.d/99-legionglow-lenovo-rgb.rules",
        );

        assert_eq!(
            argv,
            vec![
                "install",
                "-m",
                "0644",
                "/tmp/legionglow-rule.rules",
                "/etc/udev/rules.d/99-legionglow-lenovo-rgb.rules",
            ]
        );
    }

    #[test]
    fn argv_never_contains_a_password_placeholder() {
        // The password is supplied via stdin, never in the argument vector.
        let argv = install_rule_argv(Path::new("/tmp/x.rules"), "/etc/udev/rules.d/x.rules");
        assert!(argv.iter().all(|arg| !arg.contains("password")));
    }

    #[test]
    fn remove_argv_targets_only_given_path() {
        assert_eq!(
            remove_rule_argv("/etc/udev/rules.d/99-legionglow-lenovo-rgb.rules"),
            vec![
                "rm",
                "-f",
                "/etc/udev/rules.d/99-legionglow-lenovo-rgb.rules"
            ]
        );
    }

    #[test]
    fn reload_and_trigger_argv_are_stable() {
        assert_eq!(
            reload_rules_argv(),
            vec!["udevadm", "control", "--reload-rules"]
        );
        assert_eq!(trigger_argv(), vec!["udevadm", "trigger"]);
    }

    #[test]
    fn missing_sudo_has_friendly_message() {
        let message = PrivilegedError::SudoMissing.user_message();

        assert!(message.to_lowercase().contains("sudo"));
        assert!(message.to_lowercase().contains("manual"));
    }

    #[test]
    fn detects_incorrect_password_strings() {
        assert!(looks_like_incorrect_password(
            "sudo: 1 incorrect password attempt"
        ));
        assert!(looks_like_incorrect_password("Sorry, try again."));
        assert!(looks_like_incorrect_password(
            "sudo: a password is required"
        ));
        assert!(!looks_like_incorrect_password(
            "install: cannot create regular file"
        ));
    }
}
