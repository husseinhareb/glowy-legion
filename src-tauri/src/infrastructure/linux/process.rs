use std::fs;

/// Read the effective UID of the current process from `/proc/self/status`.
///
/// This avoids pulling in `libc` or any `unsafe` code. The `Uid:` line has the
/// form `Uid:\t<real>\t<effective>\t<saved>\t<fs>`; we return the effective UID.
/// Returns `None` when the value cannot be determined (e.g. non-Linux or a
/// restricted `/proc`).
pub fn effective_uid() -> Option<u32> {
    let status = fs::read_to_string("/proc/self/status").ok()?;

    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("Uid:") {
            let mut fields = rest.split_whitespace();
            let _real = fields.next();
            return fields.next().and_then(|value| value.parse::<u32>().ok());
        }
    }

    None
}

/// Whether the current process is running as root.
///
/// When the UID cannot be determined we conservatively assume non-root so that
/// permission diagnostics are not suppressed.
pub fn is_running_as_root() -> bool {
    effective_uid().map(|uid| uid == 0).unwrap_or(false)
}
