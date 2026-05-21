use std::future::Future;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use super::framing::{content_length, find_header_end};

pub(super) struct RmcpContentLengthTransport<R, W, Role> {
    read: R,
    write: Arc<tokio::sync::Mutex<W>>,
    buffer: Vec<u8>,
    max_message_bytes: usize,
    error_state: RmcpTransportErrorState,
    role: PhantomData<Role>,
}

#[derive(Clone, Default)]
pub(super) struct RmcpTransportErrorState {
    message: Arc<Mutex<Option<String>>>,
}

impl RmcpTransportErrorState {
    pub(super) fn record(&self, error: std::io::Error) {
        if let Ok(mut message) = self.message.lock() {
            *message = Some(error.to_string());
        }
    }

    pub(super) fn take(&self) -> Option<String> {
        self.message
            .lock()
            .ok()
            .and_then(|mut message| message.take())
    }
}

impl<R, W, Role> RmcpContentLengthTransport<R, W, Role> {
    pub(super) fn new(
        read: R,
        write: W,
        max_message_bytes: usize,
        error_state: RmcpTransportErrorState,
    ) -> Self {
        Self {
            read,
            write: Arc::new(tokio::sync::Mutex::new(write)),
            buffer: Vec::new(),
            max_message_bytes,
            error_state,
            role: PhantomData,
        }
    }
}

impl<R, W, Role> rmcp::transport::Transport<Role> for RmcpContentLengthTransport<R, W, Role>
where
    R: tokio::io::AsyncRead + Send + Unpin + 'static,
    W: tokio::io::AsyncWrite + Send + Unpin + 'static,
    Role: rmcp::service::ServiceRole,
{
    type Error = std::io::Error;

    fn send(
        &mut self,
        item: rmcp::service::TxJsonRpcMessage<Role>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'static {
        let write = Arc::clone(&self.write);
        async move {
            let body = serde_json::to_vec(&item).map_err(std::io::Error::other)?;
            let header = format!("Content-Length: {}\r\n\r\n", body.len());
            let mut write = write.lock().await;
            tokio::io::AsyncWriteExt::write_all(&mut *write, header.as_bytes()).await?;
            tokio::io::AsyncWriteExt::write_all(&mut *write, &body).await?;
            tokio::io::AsyncWriteExt::flush(&mut *write).await
        }
    }

    async fn receive(&mut self) -> Option<rmcp::service::RxJsonRpcMessage<Role>> {
        match next_rmcp_framed_message::<R, Role>(
            &mut self.read,
            &mut self.buffer,
            self.max_message_bytes,
        )
        .await
        {
            Ok(Some(message)) => Some(message),
            Ok(None) => None,
            Err(error) => {
                self.error_state.record(error);
                None
            }
        }
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        let mut write = self.write.lock().await;
        tokio::io::AsyncWriteExt::shutdown(&mut *write).await
    }
}

async fn next_rmcp_framed_message<R, Role>(
    read: &mut R,
    buffer: &mut Vec<u8>,
    max_message_bytes: usize,
) -> Result<Option<rmcp::service::RxJsonRpcMessage<Role>>, std::io::Error>
where
    R: tokio::io::AsyncRead + Unpin,
    Role: rmcp::service::ServiceRole,
{
    loop {
        if let Some(message) = parse_next_rmcp_framed_message::<Role>(buffer, max_message_bytes)? {
            return Ok(Some(message));
        }
        if buffer.len() > max_message_bytes {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "MCP message exceeded size limit.",
            ));
        }
        let mut chunk = [0_u8; 8192];
        let read = tokio::io::AsyncReadExt::read(read, &mut chunk).await?;
        if read == 0 {
            return Ok(None);
        }
        buffer.extend_from_slice(&chunk[..read]);
    }
}

fn parse_next_rmcp_framed_message<Role>(
    buffer: &mut Vec<u8>,
    max_message_bytes: usize,
) -> Result<Option<rmcp::service::RxJsonRpcMessage<Role>>, std::io::Error>
where
    Role: rmcp::service::ServiceRole,
{
    let Some(header_end) = find_header_end(buffer) else {
        return Ok(None);
    };
    if header_end > max_message_bytes {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "MCP message exceeded size limit.",
        ));
    }
    let header = std::str::from_utf8(&buffer[..header_end])
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    let content_length = content_length(header).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "MCP message missing Content-Length.",
        )
    })?;
    if content_length > max_message_bytes {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "MCP message exceeded size limit.",
        ));
    }
    let body_start = header_end + 4;
    let body_end = body_start.saturating_add(content_length);
    if buffer.len() < body_end {
        return Ok(None);
    }
    let body = buffer[body_start..body_end].to_vec();
    buffer.drain(..body_end);
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}
