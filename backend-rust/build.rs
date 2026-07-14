// build.rs — Injects compile-time metadata as environment variables.
// These are embedded in the binary and exposed via the /health endpoint.
// No runtime overhead: values are resolved once at compile time.

use std::process::Command;

fn main() {
    // ── Git commit SHA ────────────────────────────────────────────────────────
    // Attempt to read the short commit hash. Falls back to "unknown" if:
    // - git is not installed
    // - the build runs outside a git repository (e.g., in a Docker scratch stage)
    let git_commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=GIT_COMMIT={}", git_commit);

    // ── Build date (UTC) ──────────────────────────────────────────────────────
    // Format: YYYY-MM-DD. Uses std::time to avoid chrono dependency in build.rs.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Simple UNIX timestamp → YYYY-MM-DD conversion (no external crates)
    let days_since_epoch = now / 86400;
    let build_date = unix_days_to_date(days_since_epoch);
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // Rerun only when git HEAD changes (avoids rebuilding on every cargo invocation)
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads/");
}

/// Converts days-since-epoch to ISO-8601 date string (YYYY-MM-DD).
/// Implements the Gregorian calendar proleptic algorithm.
fn unix_days_to_date(days: u64) -> String {
    // Algorithm: http://howardhinnant.github.io/date_algorithms.html
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}
