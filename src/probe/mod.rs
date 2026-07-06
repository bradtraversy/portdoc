//! Platform probing boundary (feature 4). A `Probe` answers "what is
//! listening on this machine" in raw OS terms; the snapshot adapter
//! (feature 6) turns that into the `DevSnapshot` contract. Platforms without
//! an implementation return `None` from `platform_probe()` so the binary
//! still builds and runs everywhere.

// Consumed by features 5-6; the boundary lands one feature ahead of its
// consumers, so only the tests use it for now.
#![allow(dead_code)]

#[cfg(target_os = "linux")]
mod linux;

use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
}

#[derive(Debug, Clone)]
pub struct ListeningSocket {
    pub protocol: Protocol,
    pub local_addr: IpAddr,
    pub port: u16,
    pub pid: Option<u32>,
    pub process: Option<ProcessInfo>,
}

/// Everything but `pid` is optional: unknown owners are a first-class case.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: Option<String>,
    pub command: Option<String>,
    pub cwd: Option<PathBuf>,
    pub user: Option<String>,
    pub started_secs_ago: Option<u64>,
}

#[derive(Debug, Default)]
pub struct ProbeOutput {
    pub sockets: Vec<ListeningSocket>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("probing is not supported on this platform")]
    Unsupported,
}

pub trait Probe {
    /// Short backend name for diagnostics ("linux-proc").
    fn name(&self) -> &'static str;
    fn probe(&self) -> Result<ProbeOutput, ProbeError>;
}

#[cfg(target_os = "linux")]
pub fn platform_probe() -> Option<Box<dyn Probe>> {
    Some(Box::new(linux::LinuxProbe))
}

#[cfg(not(target_os = "linux"))]
pub fn platform_probe() -> Option<Box<dyn Probe>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn linux_gets_a_probe() {
        let probe = platform_probe().expect("linux should have a probe");
        assert_eq!(probe.name(), "linux-proc");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn skeleton_probe_returns_empty_output() {
        let probe = platform_probe().expect("linux should have a probe");
        let output = probe.probe().expect("skeleton probe should not fail");
        assert!(output.sockets.is_empty());
    }

    #[test]
    fn probe_error_formats_with_context() {
        let err = ProbeError::Io {
            path: "/proc/net/tcp".into(),
            source: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
        };
        assert!(err.to_string().contains("/proc/net/tcp"));
        assert_eq!(
            ProbeError::Unsupported.to_string(),
            "probing is not supported on this platform"
        );
    }
}
