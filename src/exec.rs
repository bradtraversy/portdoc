//! Deadline-guarded command execution for probes that shell out (docker,
//! git). A hung child is killed at the deadline so no probe can stall a
//! snapshot; any failure degrades to None, never an error.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Run a command and return its stdout, or None on spawn failure, non-zero
/// exit, or deadline expiry. Stdout is drained on a thread because a killed
/// child only unblocks the reader once its pipe closes.
pub fn run(program: &str, args: &[&str], dir: Option<&Path>, timeout: Duration) -> Option<String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    let mut child = command.spawn().ok()?;
    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = stdout.read_to_string(&mut buf);
        buf
    });

    let deadline = Instant::now() + timeout;
    let success = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.success(),
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                break false;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(25)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                break false;
            }
        }
    };

    let output = reader.join().ok()?;
    success.then_some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_stdout_of_a_successful_command() {
        let out = run("echo", &["hello"], None, Duration::from_secs(5)).expect("echo runs");
        assert_eq!(out.trim(), "hello");
    }

    #[test]
    fn failures_degrade_to_none() {
        assert!(
            run("false", &[], None, Duration::from_secs(5)).is_none(),
            "non-zero exit"
        );
        assert!(
            run("portdoc-no-such-binary", &[], None, Duration::from_secs(5)).is_none(),
            "missing binary"
        );
    }

    #[test]
    fn deadline_kills_a_hung_command() {
        let started = Instant::now();
        assert!(run("sleep", &["30"], None, Duration::from_millis(200)).is_none());
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "the deadline must cut the wait short"
        );
    }

    #[test]
    fn runs_in_the_given_directory() {
        let out =
            run("pwd", &[], Some(Path::new("/tmp")), Duration::from_secs(5)).expect("pwd runs");
        // canonicalize both sides: /tmp is a symlink to /private/tmp on macOS
        assert_eq!(
            std::fs::canonicalize(out.trim()).expect("canonicalize pwd output"),
            std::fs::canonicalize("/tmp").expect("canonicalize /tmp")
        );
    }
}
