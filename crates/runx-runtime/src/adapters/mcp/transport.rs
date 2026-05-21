// rust-style-allow: large-file because the client-side transport keeps stdio
// framing, response buffering, and bounded read/write helpers adjacent to the
// transport implementations they coordinate.
use std::future::Future;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

use runx_contracts::{JsonObject, JsonValue};
use serde_json::{self, Value as JsonWireValue};

use crate::credentials::SecretEnv;
use crate::sandbox::SandboxPlan;

use super::rmcp_content_length::{RmcpContentLengthTransport, RmcpTransportErrorState};
use super::templates::js_string;
use super::types::{
    McpListToolsRequest, McpToolCallRequest, McpToolDescriptor, McpTransport, McpTransportError,
};

const MAX_CLIENT_RESPONSE_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Default)]
pub struct FixtureMcpTransport;

impl FixtureMcpTransport {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl McpTransport for FixtureMcpTransport {
    fn call_tool(&self, request: McpToolCallRequest) -> Result<JsonValue, McpTransportError> {
        match request.tool.as_str() {
            "echo" => Ok(text_content(js_string(request.arguments.get("message")))),
            "env" => Ok(text_content(mcp_env_value(&request))),
            "fail" => Err(McpTransportError::tool_error(
                -32000,
                format!(
                    "fixture failure: {}",
                    js_string(request.arguments.get("message"))
                ),
            )),
            "sleep" => Err(McpTransportError::timeout(request.timeout)),
            "malformed-json" => Err(McpTransportError::failed("MCP server sent invalid JSON.")),
            _ => Err(McpTransportError::tool_error(-32601, "tool not found")),
        }
    }
}

fn mcp_env_value(request: &McpToolCallRequest) -> String {
    let name = js_string(request.arguments.get("name"));
    request
        .sandbox
        .env
        .get(&name)
        .cloned()
        .or_else(|| request.secret_env.get(&name).map(str::to_owned))
        .unwrap_or_default()
}

fn text_content(text: String) -> JsonValue {
    JsonValue::Object(
        [(
            "content".to_owned(),
            JsonValue::Array(vec![JsonValue::Object(
                [
                    ("type".to_owned(), JsonValue::String("text".to_owned())),
                    ("text".to_owned(), JsonValue::String(text)),
                ]
                .into(),
            )]),
        )]
        .into(),
    )
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessMcpTransport;

impl ProcessMcpTransport {
    pub fn list_tools(
        &self,
        request: McpListToolsRequest,
    ) -> Result<Vec<McpToolDescriptor>, McpTransportError> {
        list_tools_with_rmcp(request)
    }
}

impl McpTransport for ProcessMcpTransport {
    fn call_tool(&self, request: McpToolCallRequest) -> Result<JsonValue, McpTransportError> {
        call_tool_with_rmcp(request)
    }
}

fn list_tools_with_rmcp(
    request: McpListToolsRequest,
) -> Result<Vec<McpToolDescriptor>, McpTransportError> {
    block_on_rmcp(list_tools_with_rmcp_async(request))
}

fn call_tool_with_rmcp(request: McpToolCallRequest) -> Result<JsonValue, McpTransportError> {
    block_on_rmcp(call_tool_with_rmcp_async(request))
}

fn block_on_rmcp<T>(
    future: impl Future<Output = Result<T, McpTransportError>> + Send + 'static,
) -> Result<T, McpTransportError>
where
    T: Send + 'static,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        let join = thread::spawn(move || block_on_rmcp_without_context(future));
        return join
            .join()
            .map_err(|_| McpTransportError::failed("MCP client runtime thread failed."))?;
    }
    block_on_rmcp_without_context(future)
}

fn block_on_rmcp_without_context<T>(
    future: impl Future<Output = Result<T, McpTransportError>>,
) -> Result<T, McpTransportError>
where
    T: Send + 'static,
{
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|_| McpTransportError::failed("MCP client runtime initialization failed."))?
        .block_on(future)
}

async fn list_tools_with_rmcp_async(
    request: McpListToolsRequest,
) -> Result<Vec<McpToolDescriptor>, McpTransportError> {
    let mut child = spawn_tokio_mcp_server(&request.sandbox, &SecretEnv::default())?;
    drain_tokio_stderr(child.stderr.take());
    let result = tokio::time::timeout(request.timeout, async {
        let error_state = RmcpTransportErrorState::default();
        let mut service = serve_rmcp_client(&mut child, error_state.clone()).await?;
        let tools = service
            .peer()
            .list_all_tools()
            .await
            .map_err(|error| rmcp_service_error(error, &error_state))?;
        let _closed = service.close_with_timeout(Duration::from_millis(100)).await;
        tools
            .into_iter()
            .map(mcp_tool_descriptor_from_rmcp)
            .collect::<Result<Vec<_>, _>>()
    })
    .await;
    terminate_tokio_child(&mut child).await;
    match result {
        Ok(result) => result,
        Err(_) => Err(McpTransportError::timeout(request.timeout)),
    }
}

async fn call_tool_with_rmcp_async(
    request: McpToolCallRequest,
) -> Result<JsonValue, McpTransportError> {
    let mut child = spawn_tokio_mcp_server(&request.sandbox, &request.secret_env)?;
    drain_tokio_stderr(child.stderr.take());
    let timeout = request.timeout;
    let result = tokio::time::timeout(timeout, async {
        let error_state = RmcpTransportErrorState::default();
        let mut service = serve_rmcp_client(&mut child, error_state.clone()).await?;
        let arguments = rmcp_json_object(request.arguments)?;
        let result = service
            .peer()
            .call_tool(
                rmcp::model::CallToolRequestParams::new(request.tool).with_arguments(arguments),
            )
            .await
            .map_err(|error| rmcp_service_error(error, &error_state))?;
        let _closed = service.close_with_timeout(Duration::from_millis(100)).await;
        rmcp_call_tool_result_json(result)
    })
    .await;
    terminate_tokio_child(&mut child).await;
    match result {
        Ok(result) => result,
        Err(_) => Err(McpTransportError::timeout(timeout)),
    }
}

async fn serve_rmcp_client(
    child: &mut tokio::process::Child,
    error_state: RmcpTransportErrorState,
) -> Result<
    rmcp::service::RunningService<rmcp::RoleClient, rmcp::model::ClientInfo>,
    McpTransportError,
> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| McpTransportError::failed("MCP server stdout unavailable."))?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| McpTransportError::failed("MCP server stdin unavailable."))?;
    let transport = RmcpContentLengthTransport::new(
        stdout,
        stdin,
        MAX_CLIENT_RESPONSE_BYTES,
        error_state.clone(),
    );
    serve_rmcp_transport(transport, &error_state).await
}

async fn serve_rmcp_transport<T, E>(
    transport: T,
    error_state: &RmcpTransportErrorState,
) -> Result<
    rmcp::service::RunningService<rmcp::RoleClient, rmcp::model::ClientInfo>,
    McpTransportError,
>
where
    T: rmcp::transport::Transport<rmcp::RoleClient, Error = E> + Send + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    rmcp::serve_client(rmcp::model::ClientInfo::default(), transport)
        .await
        .map_err(|error| rmcp_initialization_error(error, error_state))
}

fn spawn_tokio_mcp_server(
    plan: &SandboxPlan,
    secret_env: &SecretEnv,
) -> Result<tokio::process::Child, McpTransportError> {
    let mut command = tokio::process::Command::new(&plan.command);
    command
        .args(&plan.args)
        .current_dir(&plan.cwd)
        .env_clear()
        .envs(&plan.env)
        .envs(secret_env.iter())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
        .spawn()
        .map_err(|_| McpTransportError::failed("MCP server failed to spawn."))
}

async fn terminate_tokio_child(child: &mut tokio::process::Child) {
    let _ = child.start_kill();
    let _ = child.wait().await;
}

fn drain_tokio_stderr(stderr: Option<tokio::process::ChildStderr>) {
    if let Some(mut stderr) = stderr {
        tokio::spawn(async move {
            let mut sink = [0_u8; 8192];
            let mut read_total = 0_usize;
            while read_total < MAX_CLIENT_RESPONSE_BYTES {
                match tokio::io::AsyncReadExt::read(&mut stderr, &mut sink).await {
                    Ok(0) | Err(_) => return,
                    Ok(read) => read_total = read_total.saturating_add(read),
                }
            }
        });
    }
}

fn mcp_tool_descriptor_from_rmcp(
    tool: rmcp::model::Tool,
) -> Result<McpToolDescriptor, McpTransportError> {
    Ok(McpToolDescriptor {
        name: tool.name.into_owned(),
        description: tool.description.map(std::borrow::Cow::into_owned),
        input_schema: Some(runx_json_object(JsonWireValue::Object(
            (*tool.input_schema).clone(),
        ))?),
    })
}

fn rmcp_call_tool_result_json(
    result: rmcp::model::CallToolResult,
) -> Result<JsonValue, McpTransportError> {
    let value = serde_json::to_value(result)
        .map_err(|_| McpTransportError::failed("MCP response serialization failed."))?;
    serde_json::from_value(value)
        .map_err(|_| McpTransportError::failed("MCP response conversion failed."))
}

fn rmcp_json_object(value: JsonObject) -> Result<rmcp::model::JsonObject, McpTransportError> {
    match serde_json::to_value(value)
        .map_err(|_| McpTransportError::failed("MCP request conversion failed."))?
    {
        JsonWireValue::Object(record) => Ok(record),
        _ => Err(McpTransportError::failed("MCP request conversion failed.")),
    }
}

fn runx_json_object(value: JsonWireValue) -> Result<JsonObject, McpTransportError> {
    serde_json::from_value(value)
        .map_err(|_| McpTransportError::failed("MCP response conversion failed."))
}

fn rmcp_service_error(
    error: rmcp::ServiceError,
    error_state: &RmcpTransportErrorState,
) -> McpTransportError {
    if let Some(message) = error_state.take() {
        return McpTransportError::failed(message);
    }
    match error {
        rmcp::ServiceError::McpError(error) => {
            McpTransportError::tool_error(i64::from(error.code.0), "MCP server returned error.")
        }
        _ => McpTransportError::failed("MCP server request failed."),
    }
}

fn rmcp_initialization_error(
    _error: rmcp::service::ClientInitializeError,
    error_state: &RmcpTransportErrorState,
) -> McpTransportError {
    if let Some(message) = error_state.take() {
        return McpTransportError::failed(message);
    }
    McpTransportError::failed("MCP client initialization failed.")
}

#[cfg(all(test, feature = "mcp"))]
mod rmcp_transport_tests {
    use rmcp::transport::Transport;
    use tokio::io::AsyncWriteExt;

    use super::{
        MAX_CLIENT_RESPONSE_BYTES, RmcpContentLengthTransport, RmcpTransportErrorState,
        serve_rmcp_transport,
    };

    // rust-style-allow: long-function because these adjacent transport
    // regression tests share one in-memory Content-Length fixture.
    #[test]
    fn rmcp_receive_records_malformed_json_as_transport_error() {
        let message = receive_error_message(b"Content-Length: 1\r\n\r\n{");

        assert!(
            message
                .as_deref()
                .is_some_and(|message| message.contains("EOF while parsing an object")),
            "{message:?}"
        );
    }

    #[test]
    fn rmcp_receive_records_missing_content_length_as_transport_error() {
        let message = receive_error_message(b"X-Test: true\r\n\r\n{}");

        assert_eq!(
            message.as_deref(),
            Some("MCP message missing Content-Length.")
        );
    }

    #[test]
    fn rmcp_receive_records_oversized_body_as_transport_error() {
        let message = receive_error_message(b"Content-Length: 1048577\r\n\r\n{}");

        assert_eq!(message.as_deref(), Some("MCP message exceeded size limit."));
    }

    #[test]
    fn rmcp_initialize_surfaces_recorded_transport_error() {
        let message = initialize_error_message(b"Content-Length: 1\r\n\r\n{");

        assert!(
            message
                .as_deref()
                .is_some_and(|message| message.contains("EOF while parsing an object")),
            "{message:?}"
        );
    }

    fn initialize_error_message(bytes: &'static [u8]) -> Option<String> {
        tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .ok()?
            .block_on(async move {
                let (mut writer, reader) = tokio::io::duplex(bytes.len().max(1));
                writer.write_all(bytes).await.ok()?;
                drop(writer);

                let error_state = RmcpTransportErrorState::default();
                let transport = RmcpContentLengthTransport::new(
                    reader,
                    tokio::io::sink(),
                    MAX_CLIENT_RESPONSE_BYTES,
                    error_state.clone(),
                );

                serve_rmcp_transport(transport, &error_state)
                    .await
                    .err()
                    .map(|error| error.message_for_test().to_owned())
            })
    }

    fn receive_error_message(bytes: &'static [u8]) -> Option<String> {
        tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .ok()?
            .block_on(async move {
                let (mut writer, reader) = tokio::io::duplex(bytes.len().max(1));
                writer.write_all(bytes).await.ok()?;
                drop(writer);

                let error_state = RmcpTransportErrorState::default();
                let mut transport = RmcpContentLengthTransport::new(
                    reader,
                    tokio::io::sink(),
                    MAX_CLIENT_RESPONSE_BYTES,
                    error_state.clone(),
                );

                let message = Transport::<rmcp::RoleClient>::receive(&mut transport).await;
                assert!(message.is_none());
                error_state.take()
            })
    }
}
