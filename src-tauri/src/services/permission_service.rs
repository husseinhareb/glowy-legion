use std::sync::Arc;

use crate::{
    app::error::AppError,
    domain::{HidAccessProbe, PermissionSetupResult, UdevRulePreview},
    drivers::keyboard_driver::KeyboardDriver,
    infrastructure::linux::{
        privileged::{
            install_rule_argv, reload_rules_argv, remove_rule_argv, run_with_sudo, sudo_available,
            trigger_add_argv, trigger_argv, PrivilegedError, PrivilegedOutcome,
        },
        udev::{
            manual_install_commands, manual_reload_commands, normalize_device_id, udev_rule_line,
            udev_rule_path,
        },
    },
};

const ACTION_INSTALL: &str = "install-udev-rule";
const ACTION_RELOAD: &str = "reload-udev-rules";
const ACTION_REMOVE: &str = "remove-udev-rule";

#[derive(Clone)]
pub struct PermissionService {
    driver: Arc<dyn KeyboardDriver>,
}

impl PermissionService {
    pub fn new(driver: Arc<dyn KeyboardDriver>) -> Self {
        Self { driver }
    }

    pub fn preview_udev_rule(&self) -> Result<UdevRulePreview, AppError> {
        self.driver.preview_udev_rule()
    }

    pub fn probe_hid_access(&self) -> Result<HidAccessProbe, AppError> {
        self.driver.probe_hid_access()
    }

    pub fn install_udev_rule(&self, password: &str) -> Result<PermissionSetupResult, AppError> {
        let (vendor_id, product_id) = match self.detected_ids()? {
            Some(ids) => ids,
            None => return Ok(no_device_failure(ACTION_INSTALL)),
        };

        // Defense in depth: re-validate the device-supplied ids before building
        // any command. The frontend never provides ids, rules, or filenames.
        if let Err(reason) = normalize_device_id(&format!("{vendor_id:04x}:{product_id:04x}")) {
            return Ok(PermissionSetupResult::failure(
                ACTION_INSTALL,
                format!("Refusing to build an install command: {reason}."),
            ));
        }

        if !sudo_available() {
            return Ok(no_sudo_failure(
                ACTION_INSTALL,
                "Install the udev rule",
                manual_install_commands(vendor_id, product_id),
            ));
        }

        // Write the generated rule to a temp file, then copy it into place with
        // a fixed mode via `install`. This avoids any shell quoting of contents.
        let temp_path =
            std::env::temp_dir().join(format!("legionglow-rule-{}.rules", std::process::id()));
        let rule = format!("{}\n", udev_rule_line(vendor_id, product_id));
        if let Err(error) = std::fs::write(&temp_path, rule) {
            return Ok(PermissionSetupResult::failure(
                ACTION_INSTALL,
                format!("Could not stage the rule file: {error}."),
            ));
        }

        // One authentication covers the whole bring-up: copy the rule into
        // place, reload rules, then re-emit an `add` event for this exact device
        // so a `uaccess` rule applies immediately to the already-connected
        // (non-removable) keyboard.
        let dest = udev_rule_path();
        let steps = [
            install_rule_argv(&temp_path, &dest),
            reload_rules_argv(),
            trigger_add_argv(vendor_id, product_id),
        ];
        let result = self.run_sequence(
            password,
            ACTION_INSTALL,
            &format!("Installed {dest}, reloaded rules, and re-triggered the device."),
            "Installing the udev rule failed.",
            &steps,
            true,
            vec![
                "Click \"Probe HID access\" and confirm can_open becomes true. Fixed permissions \
                 do not mean the device is safe to open; the probe stays gated on safe interface \
                 identification."
                    .to_string(),
                "If it is still blocked, log out and back in (or reboot) — an internal keyboard \
                 cannot be replugged, and uaccess applies on the next session/device add."
                    .to_string(),
                "Stay in dry-run mode until access is confirmed; real writes remain separate."
                    .to_string(),
            ],
            manual_install_commands(vendor_id, product_id),
        );
        let _ = std::fs::remove_file(&temp_path);
        Ok(result)
    }

    pub fn reload_udev_rules(&self, password: &str) -> Result<PermissionSetupResult, AppError> {
        if !sudo_available() {
            return Ok(no_sudo_failure(
                ACTION_RELOAD,
                "Reload udev rules",
                manual_reload_commands(),
            ));
        }

        // When the device is known, target the add-trigger at it so a uaccess
        // rule reapplies live; otherwise fall back to a generic trigger.
        let trigger = match self.detected_ids()? {
            Some((vendor_id, product_id)) => trigger_add_argv(vendor_id, product_id),
            None => trigger_argv(),
        };
        let steps = [reload_rules_argv(), trigger];
        Ok(self.run_sequence(
            password,
            ACTION_RELOAD,
            "Reloaded udev rules and re-triggered the device.",
            "Reloading udev rules failed.",
            &steps,
            true,
            vec![
                "Click \"Probe HID access\" to confirm can_open becomes true.".to_string(),
                "If still blocked, log out and back in (or reboot) so uaccess applies.".to_string(),
            ],
            manual_reload_commands(),
        ))
    }

    pub fn remove_udev_rule(&self, password: &str) -> Result<PermissionSetupResult, AppError> {
        if !sudo_available() {
            return Ok(no_sudo_failure(
                ACTION_REMOVE,
                "Remove the LegionGlow udev rule",
                vec![format!("sudo rm -f {}", udev_rule_path())],
            ));
        }

        let steps = [
            remove_rule_argv(&udev_rule_path()),
            reload_rules_argv(),
            trigger_argv(),
        ];
        Ok(self.run_sequence(
            password,
            ACTION_REMOVE,
            "Removed the LegionGlow udev rule and reloaded rules.",
            "Removing the LegionGlow udev rule failed.",
            &steps,
            true,
            vec![
                "Only the LegionGlow-managed rule was removed.".to_string(),
                "Non-root HID access may be blocked again until a rule is reinstalled.".to_string(),
            ],
            vec![format!("sudo rm -f {}", udev_rule_path())],
        ))
    }

    fn detected_ids(&self) -> Result<Option<(u16, u16)>, AppError> {
        self.driver.detected_hid_ids()
    }

    /// Run a fixed sequence of privileged steps with one authentication, stopping
    /// at the first failure with partial details.
    #[allow(clippy::too_many_arguments)]
    fn run_sequence(
        &self,
        password: &str,
        action: &str,
        success_message: &str,
        failure_message: &str,
        steps: &[Vec<String>],
        requires_reconnect: bool,
        next_steps: Vec<String>,
        manual_commands: Vec<String>,
    ) -> PermissionSetupResult {
        let mut stdout_acc = String::new();
        let mut stderr_acc = String::new();

        for step in steps {
            match run_one(password, step) {
                Err(error) => return privileged_failure(action, error, manual_commands),
                Ok(outcome) => {
                    append(&mut stdout_acc, &outcome.stdout);
                    append(&mut stderr_acc, &outcome.stderr);
                    if !outcome.success {
                        return command_failed(action, failure_message, &outcome, manual_commands);
                    }
                }
            }
        }

        PermissionSetupResult {
            success: true,
            action: action.to_string(),
            message: success_message.to_string(),
            stdout: non_empty(stdout_acc),
            stderr: non_empty(stderr_acc),
            requires_reconnect,
            next_steps,
            warnings: Vec::new(),
        }
    }
}

fn run_one(password: &str, argv: &[String]) -> Result<PrivilegedOutcome, PrivilegedError> {
    let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
    run_with_sudo(password, &refs)
}

fn no_device_failure(action: &str) -> PermissionSetupResult {
    let mut result = PermissionSetupResult::failure(
        action,
        "No supported Lenovo HID candidate is currently detected, so there is nothing to authorize."
            .to_string(),
    );
    result
        .next_steps
        .push("Connect the keyboard and run diagnostics, then try again.".to_string());
    result
}

fn no_sudo_failure(
    action: &str,
    intent: &str,
    manual_commands: Vec<String>,
) -> PermissionSetupResult {
    PermissionSetupResult {
        success: false,
        action: action.to_string(),
        message: format!(
            "{intent}: `sudo` is not available. Run the manual commands in a terminal instead."
        ),
        stdout: None,
        stderr: None,
        requires_reconnect: false,
        next_steps: manual_commands,
        warnings: Vec::new(),
    }
}

fn privileged_failure(
    action: &str,
    error: PrivilegedError,
    manual_commands: Vec<String>,
) -> PermissionSetupResult {
    PermissionSetupResult {
        success: false,
        action: action.to_string(),
        message: error.user_message(),
        stdout: None,
        stderr: None,
        requires_reconnect: false,
        next_steps: manual_commands,
        warnings: Vec::new(),
    }
}

fn command_failed(
    action: &str,
    message: &str,
    outcome: &PrivilegedOutcome,
    manual_commands: Vec<String>,
) -> PermissionSetupResult {
    // Distinguish a wrong password (user can retry) from a genuine command error.
    if outcome.incorrect_password {
        return PermissionSetupResult {
            success: false,
            action: action.to_string(),
            message: "Incorrect password. Please try again.".to_string(),
            stdout: None,
            stderr: None,
            requires_reconnect: false,
            next_steps: Vec::new(),
            warnings: Vec::new(),
        };
    }

    PermissionSetupResult {
        success: false,
        action: action.to_string(),
        message: message.to_string(),
        stdout: non_empty(outcome.stdout.clone()),
        stderr: non_empty(outcome.stderr.clone()),
        requires_reconnect: false,
        next_steps: manual_commands,
        warnings: Vec::new(),
    }
}

fn append(acc: &mut String, value: &str) {
    if value.is_empty() {
        return;
    }
    if !acc.is_empty() {
        acc.push('\n');
    }
    acc.push_str(value);
}

fn non_empty(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        append, command_failed, no_device_failure, no_sudo_failure, non_empty, ACTION_INSTALL,
        ACTION_REMOVE,
    };
    use crate::infrastructure::linux::{
        privileged::{install_rule_argv, PrivilegedOutcome},
        udev::udev_rule_path,
    };
    use std::path::Path;

    #[test]
    fn no_device_failure_is_unsuccessful_with_guidance() {
        let result = no_device_failure(ACTION_INSTALL);

        assert!(!result.success);
        assert_eq!(result.action, ACTION_INSTALL);
        assert!(!result.next_steps.is_empty());
    }

    #[test]
    fn no_sudo_failure_points_to_manual_commands() {
        let manual = vec!["echo example".to_string()];
        let result = no_sudo_failure(ACTION_REMOVE, "Remove the rule", manual.clone());

        assert!(!result.success);
        assert!(result.message.to_lowercase().contains("sudo"));
        assert_eq!(result.next_steps, manual);
    }

    #[test]
    fn incorrect_password_is_reported_without_leaking_output() {
        let outcome = PrivilegedOutcome {
            success: false,
            code: Some(1),
            stdout: String::new(),
            stderr: "sudo: 1 incorrect password attempt".to_string(),
            incorrect_password: true,
        };

        let result = command_failed(ACTION_INSTALL, "Install failed.", &outcome, Vec::new());

        assert!(!result.success);
        assert!(result.message.to_lowercase().contains("incorrect password"));
        // The raw sudo stderr is not surfaced for a wrong-password case.
        assert!(result.stderr.is_none());
    }

    #[test]
    fn install_command_targets_constant_path_not_frontend_input() {
        // Only a backend-owned temp file is the source; the destination is the
        // constant managed path. No frontend-provided value participates.
        let temp = Path::new("/tmp/legionglow-rule-123.rules");
        let argv = install_rule_argv(temp, &udev_rule_path());

        assert_eq!(argv.first().map(String::as_str), Some("install"));
        assert_eq!(
            argv.last().map(String::as_str),
            Some(udev_rule_path().as_str())
        );
        assert_eq!(
            udev_rule_path(),
            "/etc/udev/rules.d/99-legionglow-lenovo-rgb.rules"
        );
    }

    #[test]
    fn output_helpers_collapse_empty_values() {
        assert_eq!(non_empty(String::new()), None);
        assert_eq!(non_empty("ok".to_string()), Some("ok".to_string()));

        let mut acc = String::new();
        append(&mut acc, "");
        append(&mut acc, "first");
        append(&mut acc, "second");
        assert_eq!(acc, "first\nsecond");
    }
}
