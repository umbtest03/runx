use std::io::Read;
use std::thread::{self, JoinHandle};

use super::ProcessSupervisorError;

pub(super) type CaptureHandle = JoinHandle<std::io::Result<CapturedOutput>>;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CapturedOutput {
    pub(crate) bytes: Vec<u8>,
    pub(crate) truncated: bool,
}

pub(super) fn capture_pipe<R>(
    pipe: Option<R>,
    context: String,
    output_limit_bytes: usize,
) -> Result<CaptureHandle, ProcessSupervisorError>
where
    R: Read + Send + 'static,
{
    pipe.map(|reader| capture_stream(reader, output_limit_bytes))
        .ok_or_else(|| {
            ProcessSupervisorError::io(context, std::io::Error::other("pipe was not captured"))
        })
}

fn capture_stream<R>(mut reader: R, output_limit_bytes: usize) -> CaptureHandle
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut captured = Vec::new();
        let mut truncated = false;
        let mut buffer = [0_u8; 8192];
        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                return Ok(CapturedOutput {
                    bytes: captured,
                    truncated,
                });
            }
            let remaining = output_limit_bytes.saturating_sub(captured.len());
            if remaining > 0 {
                captured.extend_from_slice(&buffer[..count.min(remaining)]);
            }
            if count > remaining {
                truncated = true;
            }
        }
    })
}

pub(super) fn join_capture(
    handle: CaptureHandle,
    context: String,
) -> Result<CapturedOutput, ProcessSupervisorError> {
    match handle.join() {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(source)) => Err(ProcessSupervisorError::io(context, source)),
        Err(_) => Err(ProcessSupervisorError::io(
            context,
            std::io::Error::other("output reader thread failed"),
        )),
    }
}
