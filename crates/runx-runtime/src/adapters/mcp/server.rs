// rust-style-allow: large-file because the JSON-RPC dispatch loop, server
// state, tool-result builders, and host-result projections for `runx mcp
// serve` all sit on the same protocol surface.
use std::io::{Read, Write};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use runx_contracts::{JsonObject, JsonValue};

use super::rmcp_content_length::{RmcpContentLengthTransport, RmcpTransportErrorState};
use super::server_skill::{execute_mcp_server_skill, identifier_segment};
use super::types::{
    McpContent, McpHostRunResult, McpServerError, McpServerOptions, McpServerTool,
    McpServerToolBehavior, McpToolResult,
};

const MAX_SERVER_REQUEST_BYTES: usize = 4 * 1024 * 1024;

pub fn serve_mcp_json_rpc(
    input: impl Read + Send + Unpin + 'static,
    output: impl Write + Send + Unpin + 'static,
    options: McpServerOptions,
) -> Result<(), McpServerError> {
    assert_unique_server_tool_names(&options.tools)?;
    serve_mcp_json_rpc_with_rmcp(input, output, options)
}

pub fn mcp_tool_result_from_host_result(result: McpHostRunResult) -> McpToolResult {
    match result {
        McpHostRunResult::Completed {
            skill_name,
            output,
            receipt_id,
            runx,
        } => completed_mcp_tool_result(skill_name, output, receipt_id, runx),
        McpHostRunResult::NeedsAgent {
            skill_name,
            run_id,
            request_count,
            runx,
        } => needs_agent_mcp_tool_result(skill_name, run_id, request_count, runx),
        McpHostRunResult::Denied {
            skill_name,
            receipt_id,
            runx,
        } => denied_mcp_tool_result(skill_name, receipt_id, runx),
        McpHostRunResult::Escalated {
            skill_name,
            receipt_id,
            error,
            runx,
        } => escalated_mcp_tool_result(skill_name, receipt_id, error, runx),
        McpHostRunResult::Failed {
            skill_name,
            receipt_id,
            error,
            runx,
        } => failed_mcp_tool_result(skill_name, receipt_id, error, runx),
    }
}

fn completed_mcp_tool_result(
    skill_name: String,
    output: String,
    receipt_id: String,
    runx: JsonObject,
) -> McpToolResult {
    let text = if output.trim().is_empty() {
        format!("{skill_name} completed. Inspect receipt {receipt_id}.")
    } else {
        output
    };
    mcp_host_tool_result(text, runx, false)
}

fn needs_agent_mcp_tool_result(
    skill_name: String,
    run_id: String,
    request_count: usize,
    runx: JsonObject,
) -> McpToolResult {
    mcp_host_tool_result(
        format!(
            "{skill_name} needs agent input at {run_id}. Continue by rerunning the same skill with --run-id {run_id} --answers answers.json after resolving {request_count} request(s)."
        ),
        runx,
        false,
    )
}

fn denied_mcp_tool_result(
    skill_name: String,
    receipt_id: Option<String>,
    runx: JsonObject,
) -> McpToolResult {
    let text = match receipt_id {
        Some(receipt_id) => format!("{skill_name} was denied by policy (receipt {receipt_id})."),
        None => format!("{skill_name} was denied by policy."),
    };
    mcp_host_tool_result(text, runx, true)
}

fn escalated_mcp_tool_result(
    skill_name: String,
    receipt_id: String,
    error: String,
    runx: JsonObject,
) -> McpToolResult {
    mcp_host_tool_result(
        format!("{skill_name} escalated. Inspect receipt {receipt_id}. {error}")
            .trim()
            .to_owned(),
        runx,
        true,
    )
}

fn failed_mcp_tool_result(
    skill_name: String,
    receipt_id: Option<String>,
    error: String,
    runx: JsonObject,
) -> McpToolResult {
    mcp_host_tool_result(
        format!(
            "{skill_name} failed. Inspect receipt {}. {error}",
            receipt_id.unwrap_or_else(|| "n/a".to_owned())
        )
        .trim()
        .to_owned(),
        runx,
        true,
    )
}

fn mcp_host_tool_result(text: String, runx: JsonObject, is_error: bool) -> McpToolResult {
    McpToolResult {
        content: vec![McpContent { text }],
        structured_content: Some(runx_content(runx)),
        is_error,
    }
}

fn runx_content(runx: JsonObject) -> JsonObject {
    [("runx".to_owned(), JsonValue::Object(runx))].into()
}

fn serve_mcp_json_rpc_with_rmcp(
    input: impl Read + Send + Unpin + 'static,
    output: impl Write + Send + Unpin + 'static,
    options: McpServerOptions,
) -> Result<(), McpServerError> {
    block_on_rmcp_server(input, output, options)
}

fn block_on_rmcp_server<R, W>(
    input: R,
    output: W,
    options: McpServerOptions,
) -> Result<(), McpServerError>
where
    R: Read + Send + Unpin + 'static,
    W: Write + Send + Unpin + 'static,
{
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|error| {
            McpServerError::new(format!("MCP server runtime initialization failed: {error}"))
        })?
        .block_on(serve_mcp_json_rpc_with_rmcp_async(input, output, options))
}

async fn serve_mcp_json_rpc_with_rmcp_async<R, W>(
    input: R,
    output: W,
    options: McpServerOptions,
) -> Result<(), McpServerError>
where
    R: Read + Send + Unpin + 'static,
    W: Write + Send + Unpin + 'static,
{
    let error_state = RmcpTransportErrorState::default();
    let transport = RmcpContentLengthTransport::new(
        BlockingAsyncRead::new(input),
        BlockingAsyncWrite::new(output),
        MAX_SERVER_REQUEST_BYTES,
        error_state.clone(),
    );
    let service = RmcpProofServer {
        state: Mutex::new(McpServerState::new(options)),
    };
    let running = rmcp::serve_server(service, transport)
        .await
        .map_err(|error| {
            McpServerError::new(format!(
                "MCP rmcp server initialization failed: {}",
                error_state.take().unwrap_or_else(|| error.to_string())
            ))
        })?;
    running.waiting().await.map(|_reason| ()).map_err(|error| {
        McpServerError::new(format!(
            "MCP rmcp server task failed: {}",
            error_state.take().unwrap_or_else(|| error.to_string())
        ))
    })
}

struct RmcpProofServer {
    state: Mutex<McpServerState>,
}

impl rmcp::ServerHandler for RmcpProofServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        let (package_name, package_version) = self.state.lock().map_or_else(
            |_| ("runx-mcp".to_owned(), "0.0.0".to_owned()),
            |state| {
                (
                    state.options.package_name.clone(),
                    state.options.package_version.clone(),
                )
            },
        );
        rmcp::model::ServerInfo::new(
            rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_protocol_version(rmcp::model::ProtocolVersion::V_2025_06_18)
        .with_server_info(rmcp::model::Implementation::new(
            package_name,
            package_version,
        ))
    }

    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<rmcp::model::ListToolsResult, rmcp::ErrorData>> + Send + '_
    {
        let result = self
            .state
            .lock()
            .map_err(|_| rmcp_internal_error("MCP server state lock failed."))
            .and_then(|state| {
                let tools = state
                    .options
                    .tools
                    .iter()
                    .map(rmcp_tool_from_server_tool)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rmcp::model::ListToolsResult::with_all_items(tools))
            });
        std::future::ready(result)
    }

    fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<rmcp::model::CallToolResult, rmcp::ErrorData>> + Send + '_
    {
        let result = self
            .state
            .lock()
            .map_err(|_| rmcp_internal_error("MCP server state lock failed."))
            .and_then(|mut state| {
                let arguments = match request.arguments {
                    Some(arguments) => runx_json_object(arguments).map_err(rmcp_invalid_params)?,
                    None => JsonObject::new(),
                };
                handle_rmcp_tool_call(&mut state, &request.name, arguments)
            });
        std::future::ready(result)
    }

    fn get_tool(&self, name: &str) -> Option<rmcp::model::Tool> {
        self.state
            .lock()
            .ok()
            .and_then(|state| {
                state
                    .options
                    .tools
                    .iter()
                    .find(|tool| tool.name == name)
                    .cloned()
            })
            .and_then(|tool| rmcp_tool_from_server_tool(&tool).ok())
    }
}

fn handle_rmcp_tool_call(
    state: &mut McpServerState,
    name: &str,
    arguments: JsonObject,
) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
    let Some(tool) = state.options.tools.iter().find(|tool| tool.name == name) else {
        return Err(rmcp::ErrorData::new(
            rmcp::model::ErrorCode::METHOD_NOT_FOUND,
            format!("tool not found: {name}"),
            None,
        ));
    };
    match tool.result.clone() {
        McpServerToolBehavior::Fixed(result) => rmcp_call_tool_result(result),
        McpServerToolBehavior::Skill(execution) => {
            match execute_mcp_server_skill(state, *execution, arguments) {
                Ok(result) => rmcp_call_tool_result(result),
                Err(error) => Err(rmcp_internal_error(error.to_string())),
            }
        }
    }
}

fn rmcp_tool_from_server_tool(tool: &McpServerTool) -> Result<rmcp::model::Tool, rmcp::ErrorData> {
    Ok(rmcp::model::Tool::new(
        tool.name.clone(),
        tool.description.clone(),
        Arc::new(rmcp_json_object(tool.input_schema.clone())?),
    ))
}

fn rmcp_call_tool_result(
    result: McpToolResult,
) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
    let content = result
        .content
        .into_iter()
        .map(|entry| rmcp::model::Content::text(entry.text))
        .collect();
    let mut call_result = if result.is_error {
        rmcp::model::CallToolResult::error(content)
    } else {
        rmcp::model::CallToolResult::success(content)
    };
    call_result.structured_content = result
        .structured_content
        .map(|content| serde_json::to_value(content).map_err(rmcp_internal_error))
        .transpose()?;
    Ok(call_result)
}

fn rmcp_json_object(value: JsonObject) -> Result<rmcp::model::JsonObject, rmcp::ErrorData> {
    match serde_json::to_value(JsonValue::Object(value)).map_err(rmcp_internal_error)? {
        serde_json::Value::Object(object) => Ok(object),
        _ => Err(rmcp_internal_error(
            "MCP tool input schema did not serialize to a JSON object.",
        )),
    }
}

fn runx_json_object(value: rmcp::model::JsonObject) -> Result<JsonObject, serde_json::Error> {
    serde_json::to_vec(&value).and_then(|bytes| serde_json::from_slice(&bytes))
}

fn rmcp_invalid_params(error: impl std::fmt::Display) -> rmcp::ErrorData {
    rmcp::ErrorData::invalid_params(error.to_string(), None)
}

fn rmcp_internal_error(error: impl std::fmt::Display) -> rmcp::ErrorData {
    rmcp::ErrorData::internal_error(error.to_string(), None)
}

struct BlockingAsyncRead<R> {
    inner: R,
}

impl<R> BlockingAsyncRead<R> {
    fn new(inner: R) -> Self {
        Self { inner }
    }
}

impl<R> tokio::io::AsyncRead for BlockingAsyncRead<R>
where
    R: Read + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        let read = self.inner.read(buf.initialize_unfilled())?;
        buf.advance(read);
        Poll::Ready(Ok(()))
    }
}

struct BlockingAsyncWrite<W> {
    inner: W,
}

impl<W> BlockingAsyncWrite<W> {
    fn new(inner: W) -> Self {
        Self { inner }
    }
}

impl<W> tokio::io::AsyncWrite for BlockingAsyncWrite<W>
where
    W: Write + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Poll::Ready(self.inner.write(buf))
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(self.inner.flush())
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(self.inner.flush())
    }
}

#[derive(Debug)]
pub(super) struct McpServerState {
    options: McpServerOptions,
    next_run_sequence: u64,
}

impl McpServerState {
    fn new(options: McpServerOptions) -> Self {
        Self {
            options,
            next_run_sequence: 0,
        }
    }

    pub(super) fn next_run_id(&mut self, skill_name: &str) -> String {
        self.next_run_sequence = self.next_run_sequence.saturating_add(1);
        format!(
            "rx_mcp_{}_{}",
            identifier_segment(skill_name),
            self.next_run_sequence
        )
    }
}

fn assert_unique_server_tool_names(tools: &[McpServerTool]) -> Result<(), McpServerError> {
    let mut seen = std::collections::BTreeSet::new();
    for tool in tools {
        if !seen.insert(tool.name.as_str()) {
            return Err(McpServerError::new(format!(
                "runx mcp serve received duplicate tool name '{}'. Serve unique skill names only.",
                tool.name
            )));
        }
    }
    Ok(())
}
