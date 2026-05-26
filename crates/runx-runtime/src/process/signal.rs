use std::process::{Child, Command};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use super::{ProcessSpec, ProcessSupervisorError, kill_timed_out_context, poll_timed_out_context};

#[derive(Clone, Copy, Debug)]
pub(crate) enum ProcessSignal {
    Terminate,
    Force,
}

#[cfg(unix)]
impl ProcessSignal {
    const fn rustix_signal(self) -> rustix::process::Signal {
        match self {
            Self::Terminate => rustix::process::Signal::TERM,
            Self::Force => rustix::process::Signal::KILL,
        }
    }
}

#[cfg(unix)]
pub(crate) fn configure_process_group(command: &mut Command) {
    command.process_group(0);
}

#[cfg(not(unix))]
pub(crate) fn configure_process_group(_command: &mut Command) {}

#[cfg(unix)]
pub(super) fn signal_timed_out_process(
    child: &mut Child,
    signal: ProcessSignal,
    spec: &ProcessSpec,
) -> Result<(), ProcessSupervisorError> {
    if signal_process_group_id(child.id(), signal) {
        return Ok(());
    }
    if child
        .try_wait()
        .map_err(|source| ProcessSupervisorError::io(poll_timed_out_context(spec.label), source))?
        .is_some()
    {
        return Ok(());
    }
    kill_direct_child_if_running(child, spec)
}

#[cfg(not(unix))]
pub(super) fn signal_timed_out_process(
    child: &mut Child,
    _signal: ProcessSignal,
    spec: &ProcessSpec,
) -> Result<(), ProcessSupervisorError> {
    kill_direct_child_if_running(child, spec)
}

#[cfg(unix)]
pub(crate) fn signal_process_group_id(process_id: u32, signal: ProcessSignal) -> bool {
    use rustix::process::{Pid, kill_process_group};

    let Ok(raw_pid) = i32::try_from(process_id) else {
        return false;
    };
    let Some(pid) = Pid::from_raw(raw_pid) else {
        return false;
    };
    kill_process_group(pid, signal.rustix_signal()).is_ok()
}

#[cfg(not(unix))]
pub(crate) fn signal_process_group_id(_process_id: u32, _signal: ProcessSignal) -> bool {
    false
}

fn kill_direct_child_if_running(
    child: &mut Child,
    spec: &ProcessSpec,
) -> Result<(), ProcessSupervisorError> {
    if child
        .try_wait()
        .map_err(|source| ProcessSupervisorError::io(poll_timed_out_context(spec.label), source))?
        .is_some()
    {
        return Ok(());
    }
    child
        .kill()
        .map_err(|source| ProcessSupervisorError::io(kill_timed_out_context(spec.label), source))
}
