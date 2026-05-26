use std::process::{Child, Command};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use super::{ProcessSpec, ProcessSupervisorError, kill_timed_out_context, poll_timed_out_context};

#[derive(Clone, Copy, Debug)]
pub(super) enum ProcessSignal {
    Terminate,
    Force,
}

#[cfg(unix)]
pub(super) fn configure_process_group(command: &mut Command) {
    command.process_group(0);
}

#[cfg(not(unix))]
pub(super) fn configure_process_group(_command: &mut Command) {}

#[cfg(unix)]
pub(super) fn signal_timed_out_process(
    child: &mut Child,
    signal: ProcessSignal,
    spec: &ProcessSpec,
) -> Result<(), ProcessSupervisorError> {
    use rustix::process::{Pid, Signal, kill_process_group};

    let pid = Pid::from_child(child);
    let signal = match signal {
        ProcessSignal::Terminate => Signal::TERM,
        ProcessSignal::Force => Signal::KILL,
    };
    if kill_process_group(pid, signal).is_ok() {
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
