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

    // Per-OS command fixtures so every test runs on the whole CI matrix.
    #[cfg(unix)]
    const ECHO_HELLO: (&str, &[&str]) = ("echo", &["hello"]);
    #[cfg(windows)]
    const ECHO_HELLO: (&str, &[&str]) = ("cmd", &["/C", "echo hello"]);

    #[cfg(unix)]
    const EXIT_NONZERO: (&str, &[&str]) = ("false", &[]);
    #[cfg(windows)]
    const EXIT_NONZERO: (&str, &[&str]) = ("cmd", &["/C", "exit 1"]);

    #[cfg(unix)]
    const HANG_30S: (&str, &[&str]) = ("sleep", &["30"]);
    // ping waits a second between its 31 probes - the Windows sleep idiom
    #[cfg(windows)]
    const HANG_30S: (&str, &[&str]) = ("ping", &["-n", "31", "127.0.0.1"]);

    #[cfg(unix)]
    const PRINT_CWD: (&str, &[&str]) = ("pwd", &[]);
    #[cfg(windows)]
    const PRINT_CWD: (&str, &[&str]) = ("cmd", &["/C", "cd"]);

    #[test]
    fn captures_stdout_of_a_successful_command() {
        let (program, args) = ECHO_HELLO;
        let out = run(program, args, None, Duration::from_secs(5)).expect("echo runs");
        assert_eq!(out.trim(), "hello");
    }

    #[test]
    fn failures_degrade_to_none() {
        let (program, args) = EXIT_NONZERO;
        assert!(
            run(program, args, None, Duration::from_secs(5)).is_none(),
            "non-zero exit"
        );
        assert!(
            run("portdoc-no-such-binary", &[], None, Duration::from_secs(5)).is_none(),
            "missing binary"
        );
    }

    #[test]
    fn deadline_kills_a_hung_command() {
        let (program, args) = HANG_30S;
        let started = Instant::now();
        assert!(run(program, args, None, Duration::from_millis(200)).is_none());
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "the deadline must cut the wait short"
        );
    }

    #[test]
    fn runs_in_the_given_directory() {
        let dir = std::env::temp_dir();
        let (program, args) = PRINT_CWD;
        let out = run(program, args, Some(&dir), Duration::from_secs(5)).expect("print cwd runs");
        // canonicalize both sides: temp dirs hide behind symlinks on macOS
        // (/tmp -> /private/tmp) and UNC prefixes on Windows
        assert_eq!(
            std::fs::canonicalize(out.trim()).expect("canonicalize cwd output"),
            std::fs::canonicalize(&dir).expect("canonicalize temp dir")
        );
    }
}
