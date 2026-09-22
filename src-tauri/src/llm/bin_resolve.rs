//! Find a CLI without trusting PATH.
//!
//! A GUI app does not inherit a login shell. Launched from Finder, Spotlight,
//! or a `nootle://` URL, the process gets `/usr/bin:/bin:/usr/sbin:/sbin` and
//! nothing else -- so `Command::new("codex")` fails even though the same
//! binary runs fine from a terminal.
//!
//! That failure is silent and total: no provider registers, so a recording
//! transcribes and then produces no summary, no insights and no export, while
//! the meeting is still marked "summarized". It only shows up in production,
//! because every developer launches the app from a shell that has the full
//! PATH already.
//!
//! So: look where these tools actually install, then fall back to asking a
//! login shell, and only then give up.

use std::path::PathBuf;

/// Where user-level CLIs land on macOS, most specific first.
fn candidate_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        dirs.push(home.join(".local/bin"));
        dirs.push(home.join("bin"));
        dirs.push(home.join(".bun/bin"));
        dirs.push(home.join(".cargo/bin"));
        dirs.push(home.join(".volta/bin"));
    }
    dirs.push(PathBuf::from("/opt/homebrew/bin"));
    dirs.push(PathBuf::from("/usr/local/bin"));
    dirs.push(PathBuf::from("/usr/bin"));
    dirs.push(PathBuf::from("/bin"));
    dirs
}

/// Absolute path to `name`, or None.
pub fn resolve(name: &str) -> Option<String> {
    // An explicit override always wins, e.g. NOOTLE_CODEX_BIN=/somewhere/codex
    let env_key = format!("NOOTLE_{}_BIN", name.to_uppercase().replace('-', "_"));
    if let Some(p) = std::env::var_os(&env_key) {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Some(p.to_string_lossy().into_owned());
        }
    }

    for dir in candidate_dirs() {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }

    // Last resort: a login shell knows about version managers and anything
    // else the user put on PATH in their profile.
    let out = std::process::Command::new("/bin/sh")
        .arg("-lc")
        .arg(format!("command -v {name}"))
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if path.is_empty() || !PathBuf::from(&path).is_file() {
        return None;
    }
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_something_that_always_exists() {
        // /bin/ls is in the candidate list, so this must not depend on PATH.
        assert_eq!(resolve("ls").as_deref(), Some("/bin/ls"));
    }

    #[test]
    fn missing_binary_is_none() {
        assert!(resolve("nootle-definitely-not-a-real-binary").is_none());
    }

    #[test]
    fn env_override_wins() {
        std::env::set_var("NOOTLE_LS_BIN", "/bin/ls");
        assert_eq!(resolve("ls").as_deref(), Some("/bin/ls"));
        std::env::remove_var("NOOTLE_LS_BIN");
    }
}
