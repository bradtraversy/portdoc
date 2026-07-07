//! Stopping services (feature 12): signal delivery and release polling.
//! The safety guards (self-refusal, verify handshake) live in the stop
//! endpoint; this module only knows pids and signals.

use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum StopError {
    #[error("permission denied - the process is owned by another user")]
    NotPermitted,
    #[error("no such process")]
    NoSuchProcess,
    #[error("failed to signal the process: {0}")]
    Io(#[from] std::io::Error),
    #[cfg(not(unix))]
    #[error("stopping processes is not supported on this platform")]
    Unsupported,
}

#[cfg(unix)]
pub fn terminate(pid: u32, force: bool) -> Result<(), StopError> {
    let signal = if force { libc::SIGKILL } else { libc::SIGTERM };
    // SAFETY: kill(2) takes a pid and a signal number; no memory involved.
    let rc = unsafe { libc::kill(pid as libc::pid_t, signal) };
    if rc == 0 {
        return Ok(());
    }
    let err = std::io::Error::last_os_error();
    match err.raw_os_error() {
        Some(libc::EPERM) => Err(StopError::NotPermitted),
        Some(libc::ESRCH) => Err(StopError::NoSuchProcess),
        _ => Err(StopError::Io(err)),
    }
}

#[cfg(not(unix))]
pub fn terminate(_pid: u32, _force: bool) -> Result<(), StopError> {
    Err(StopError::Unsupported)
}

/// Poll until the listener is gone or attempts run out. Returns true when
/// it released.
pub fn wait_released(
    mut still_listening: impl FnMut() -> bool,
    attempts: u32,
    interval: Duration,
) -> bool {
    for _ in 0..attempts {
        if !still_listening() {
            return true;
        }
        std::thread::sleep(interval);
    }
    !still_listening()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Child, Command};

    /// Bounded reap so a failed signal cannot hang the suite.
    fn exits_within(child: &mut Child, tries: u32) -> bool {
        for _ in 0..tries {
            if child.try_wait().expect("try_wait").is_some() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        false
    }

    #[test]
    fn sigterm_stops_a_cooperative_child() {
        let mut child = Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("spawn sleep");
        terminate(child.id(), false).expect("terminate should signal");
        assert!(exits_within(&mut child, 20), "child should exit on TERM");
    }

    #[test]
    fn sigkill_stops_a_term_ignoring_child() {
        let mut child = Command::new("sh")
            .args(["-c", r#"trap "" TERM; sleep 30"#])
            .spawn()
            .expect("spawn trap child");
        // let the shell install its trap before signaling
        std::thread::sleep(Duration::from_millis(200));

        terminate(child.id(), false).expect("TERM should deliver");
        std::thread::sleep(Duration::from_millis(300));
        assert!(
            child.try_wait().expect("try_wait").is_none(),
            "child ignores TERM and should still be alive"
        );

        terminate(child.id(), true).expect("KILL should deliver");
        assert!(exits_within(&mut child, 20), "child cannot ignore KILL");
    }

    #[test]
    fn nonexistent_pid_is_no_such_process() {
        // far above any default pid_max, so it cannot exist
        let err = terminate(0x3FF_FFFF, false).expect_err("must fail");
        assert!(matches!(err, StopError::NoSuchProcess));
    }

    #[test]
    fn signaling_init_is_not_permitted() {
        // SAFETY-adjacent guard: only meaningful (and only safe) as non-root.
        // SAFETY: geteuid has no preconditions.
        if unsafe { libc::geteuid() } == 0 {
            return;
        }
        let err = terminate(1, false).expect_err("non-root cannot signal init");
        assert!(matches!(err, StopError::NotPermitted));
    }

    #[test]
    fn wait_released_polls_until_released_or_gives_up() {
        let mut calls = 0;
        let released = wait_released(
            || {
                calls += 1;
                calls < 3
            },
            5,
            Duration::from_millis(1),
        );
        assert!(released);
        assert_eq!(calls, 3);

        assert!(
            !wait_released(|| true, 3, Duration::from_millis(1)),
            "never releasing exhausts attempts"
        );
    }
}
