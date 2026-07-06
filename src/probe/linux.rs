use super::{Probe, ProbeError, ProbeOutput};

/// Linux probe backed by /proc. Feature 5 implements the real socket and
/// process reading; until then it reports an empty, valid probe result.
pub struct LinuxProbe;

impl Probe for LinuxProbe {
    fn name(&self) -> &'static str {
        "linux-proc"
    }

    fn probe(&self) -> Result<ProbeOutput, ProbeError> {
        Ok(ProbeOutput::default())
    }
}
