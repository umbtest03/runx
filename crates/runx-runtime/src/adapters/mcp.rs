// rust-style-allow: large-file because the MCP slice keeps the adapter,
// fixture transport, bounded stdio framing, argument mapping, and sanitization
// beside each other until server routing introduces natural module boundaries.
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use runx_contracts::{JsonNumber, JsonObject, JsonValue};
use runx_parser::SkillMcpServer;
use sha2::{Digest, Sha256};

use crate::RuntimeError;
use crate::adapter::{InvocationStatus, SkillAdapter, SkillInvocation, SkillOutput};
use crate::sandbox::{SandboxPlan, prepare_mcp_process_sandbox, sandbox_metadata};

const DEFAULT_TIMEOUT_MS: u64 = 60_000;
const MIN_TIMEOUT_MS: u64 = 50;
const MAX_CLIENT_RESPONSE_BYTES: usize = 1024 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const PROTOCOL_VERSION: &str = "2025-06-18";
const TEMPLATE_OPEN: &str = "\x7b\x7b";
const TEMPLATE_CLOSE: &str = "\x7d\x7d";

#[derive(Clone, Debug)]
pub struct McpAdapter<T = ProcessMcpTransport> {
    transport: T,
}

impl<T> McpAdapter<T> {
    #[must_use]
    pub const fn new(transport: T) -> Self {
        Self { transport }
    }
}

impl Default for McpAdapter<ProcessMcpTransport> {
    fn default() -> Self {
        Self::new(ProcessMcpTransport)
    }
}

impl<T> SkillAdapter for McpAdapter<T>
where
    T: McpTransport,
{
    fn adapter_type(&self) -> &'static str {
        "mcp"
    }

    fn invoke(&self, request: SkillInvocation) -> Result<SkillOutput, RuntimeError> {
        let started = Instant::now();
        let prepared = match prepare_mcp_tool_call(request, started)? {
            Ok(prepared) => prepared,
            Err(output) => return Ok(output),
        };
        let metadata = prepared.metadata;
        match self.transport.call_tool(prepared.request) {
            Ok(result) => Ok(SkillOutput {
                status: InvocationStatus::Success,
                stdout: stringify_mcp_tool_result(&result)?,
                stderr: String::new(),
                exit_code: Some(0),
                duration_ms: duration_ms(started),
                metadata,
            }),
            Err(error) => Ok(failure(error.sanitized_message(), started, metadata)),
        }
    }
}

#[derive(Debug)]
struct PreparedMcpToolCall {
    request: McpToolCallRequest,
    metadata: JsonObject,
}

#[derive(Clone, Debug, PartialEq)]
pub struct McpToolCallRequest {
    pub server: SkillMcpServer,
    pub tool: String,
    pub arguments: JsonObject,
    pub timeout: Duration,
    pub sandbox: SandboxPlan,
}

pub trait McpTransport {
    fn call_tool(&self, request: McpToolCallRequest) -> Result<JsonValue, McpTransportError>;
}

impl<T> McpTransport for &T
where
    T: McpTransport + ?Sized,
{
    fn call_tool(&self, request: McpToolCallRequest) -> Result<JsonValue, McpTransportError> {
        (**self).call_tool(request)
    }
}

fn prepare_mcp_tool_call(
    invocation: SkillInvocation,
    started: Instant,
) -> Result<Result<PreparedMcpToolCall, SkillOutput>, RuntimeError> {
    let SkillInvocation {
        source,
        inputs,
        resolved_inputs,
        skill_directory,
        env,
        ..
    } = invocation;
    if source.source_type != "mcp" {
        return Err(RuntimeError::UnsupportedAdapter {
            adapter_type: source.source_type,
        });
    }
    let Some(server) = source.server.clone() else {
        return Ok(Err(missing_mcp_metadata(started)));
    };
    let Some(tool) = source.tool.clone().filter(|tool| !tool.is_empty()) else {
        return Ok(Err(missing_mcp_metadata(started)));
    };
    let arguments = map_mcp_arguments(source.arguments.as_ref(), &inputs, &resolved_inputs)?;
    let sandbox = match prepare_mcp_process_sandbox(&source, &server, &skill_directory, &env) {
        Ok(plan) => plan,
        Err(RuntimeError::SandboxViolation { message }) => {
            return Ok(Err(failure(
                format!("MCP sandbox denied: {message}"),
                started,
                metadata_for(&source, Some(sandbox_metadata(source.sandbox.as_ref())))?,
            )));
        }
        Err(error) => return Err(error),
    };
    let metadata = metadata_for(&source, Some(sandbox.metadata.clone()))?;
    Ok(Ok(PreparedMcpToolCall {
        request: McpToolCallRequest {
            server,
            tool,
            arguments,
            timeout: timeout_from_source(source.timeout_seconds),
            sandbox,
        },
        metadata,
    }))
}

fn missing_mcp_metadata(started: Instant) -> SkillOutput {
    failure(
        "MCP source requires server and tool metadata.",
        started,
        JsonObject::new(),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpTransportError {
    kind: McpTransportErrorKind,
    message: String,
}

impl McpTransportError {
    #[must_use]
    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            kind: McpTransportErrorKind::Failed,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn tool_error(code: i64, message: impl Into<String>) -> Self {
        Self {
            kind: McpTransportErrorKind::ToolError(code),
            message: message.into(),
        }
    }

    #[must_use]
    pub fn timeout(timeout: Duration) -> Self {
        let timeout_ms = u64::try_from(timeout.as_millis()).unwrap_or(u64::MAX);
        Self {
            kind: McpTransportErrorKind::Timeout,
            message: format!("MCP call timed out after {timeout_ms}ms."),
        }
    }

    #[must_use]
    pub fn sanitized_message(&self) -> String {
        match self.kind {
            McpTransportErrorKind::ToolError(code) => {
                format!("MCP tool returned error {code}.")
            }
            McpTransportErrorKind::Timeout => self.message.clone(),
            McpTransportErrorKind::Failed => "MCP adapter failed.".to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum McpTransportErrorKind {
    ToolError(i64),
    Timeout,
    Failed,
}

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
            "env" => Ok(text_content(env_value(
                &request.sandbox.env,
                request.arguments.get("name"),
            ))),
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

#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessMcpTransport;

impl McpTransport for ProcessMcpTransport {
    fn call_tool(&self, request: McpToolCallRequest) -> Result<JsonValue, McpTransportError> {
        let mut child = spawn_mcp_server(&request.sandbox)?;
        let Some(mut stdin) = child.stdin.take() else {
            terminate_child(&mut child);
            return Err(McpTransportError::failed("MCP server stdin unavailable."));
        };
        let Some(stdout) = child.stdout.take() else {
            terminate_child(&mut child);
            return Err(McpTransportError::failed("MCP server stdout unavailable."));
        };
        drain_stderr(child.stderr.take());
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || read_stdout_frames(stdout, tx));
        let deadline = Instant::now() + request.timeout;

        write_message(&mut stdin, &initialize_request(1))?;
        wait_for_response(&mut child, &rx, 1, deadline, request.timeout)?;
        write_message(&mut stdin, &initialized_notification())?;
        write_message(
            &mut stdin,
            &tool_call_request(2, &request.tool, &request.arguments),
        )?;
        let result = wait_for_response(&mut child, &rx, 2, deadline, request.timeout);
        terminate_child(&mut child);
        result
    }
}

pub fn map_mcp_arguments(
    argument_template: Option<&JsonObject>,
    inputs: &JsonObject,
    resolved_inputs: &JsonObject,
) -> Result<JsonObject, RuntimeError> {
    let Some(template) = argument_template else {
        let mut merged = inputs.clone();
        merged.extend(resolved_inputs.clone());
        return Ok(merged);
    };
    template
        .iter()
        .map(|(key, value)| {
            let mapped = match value {
                JsonValue::String(template) => {
                    map_template_string(template, inputs, resolved_inputs)?
                }
                other => other.clone(),
            };
            Ok((key.clone(), mapped))
        })
        .collect()
}

pub fn stringify_mcp_tool_result(result: &JsonValue) -> Result<String, RuntimeError> {
    if let JsonValue::Object(record) = result
        && let Some(JsonValue::Array(content)) = record.get("content")
    {
        return content
            .iter()
            .map(stringify_content_entry)
            .collect::<Result<Vec<_>, _>>()
            .map(|entries| entries.join("\n"));
    }

    match result {
        JsonValue::String(value) => Ok(value.clone()),
        value => serde_json::to_string(value)
            .map_err(|source| RuntimeError::json("serializing MCP tool result", source)),
    }
}

fn map_template_string(
    template: &str,
    inputs: &JsonObject,
    resolved_inputs: &JsonObject,
) -> Result<JsonValue, RuntimeError> {
    if let Some(key) = exact_template_key(template) {
        return Ok(resolved_inputs
            .get(key)
            .or_else(|| inputs.get(key))
            .cloned()
            .unwrap_or(JsonValue::Null));
    }

    let mut rendered = String::new();
    let mut rest = template;
    while let Some(start) = rest.find(TEMPLATE_OPEN) {
        let (prefix, after_start) = rest.split_at(start);
        rendered.push_str(prefix);
        let after_start = &after_start[2..];
        let Some(end) = after_start.find(TEMPLATE_CLOSE) else {
            rendered.push_str(TEMPLATE_OPEN);
            rendered.push_str(after_start);
            return Ok(JsonValue::String(rendered));
        };
        let raw_key = &after_start[..end];
        let key = raw_key.trim();
        if valid_template_key(key) {
            rendered.push_str(&stringify_mcp_input(
                resolved_inputs.get(key).or_else(|| inputs.get(key)),
            )?);
        } else {
            rendered.push_str(TEMPLATE_OPEN);
            rendered.push_str(raw_key);
            rendered.push_str(TEMPLATE_CLOSE);
        }
        rest = &after_start[end + 2..];
    }
    rendered.push_str(rest);
    Ok(JsonValue::String(rendered))
}

fn exact_template_key(template: &str) -> Option<&str> {
    let trimmed = template.trim();
    let inner = trimmed
        .strip_prefix(TEMPLATE_OPEN)?
        .strip_suffix(TEMPLATE_CLOSE)?
        .trim();
    if valid_template_key(inner) {
        Some(inner)
    } else {
        None
    }
}

fn valid_template_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-'))
}

fn stringify_mcp_input(value: Option<&JsonValue>) -> Result<String, RuntimeError> {
    match value {
        None | Some(JsonValue::Null) => Ok(String::new()),
        Some(JsonValue::String(value)) => Ok(value.clone()),
        Some(value) => serde_json::to_string(value)
            .map_err(|source| RuntimeError::json("serializing MCP template input", source)),
    }
}

fn stringify_content_entry(entry: &JsonValue) -> Result<String, RuntimeError> {
    if let JsonValue::Object(record) = entry
        && record.get("type") == Some(&JsonValue::String("text".to_owned()))
        && let Some(JsonValue::String(text)) = record.get("text")
    {
        return Ok(text.clone());
    }
    serde_json::to_string(entry)
        .map_err(|source| RuntimeError::json("serializing MCP content entry", source))
}

fn spawn_mcp_server(plan: &SandboxPlan) -> Result<Child, McpTransportError> {
    Command::new(&plan.command)
        .args(&plan.args)
        .current_dir(&plan.cwd)
        .env_clear()
        .envs(&plan.env)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| McpTransportError::failed("MCP server failed to spawn."))
}

fn write_message(stdin: &mut impl Write, message: &JsonValue) -> Result<(), McpTransportError> {
    let body = serde_json::to_vec(message)
        .map_err(|_| McpTransportError::failed("MCP request serialization failed."))?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    stdin
        .write_all(header.as_bytes())
        .and_then(|()| stdin.write_all(&body))
        .map_err(|_| McpTransportError::failed("MCP server stdin write failed."))
}

fn wait_for_response(
    child: &mut Child,
    rx: &Receiver<Result<JsonValue, McpTransportError>>,
    id: i64,
    deadline: Instant,
    timeout: Duration,
) -> Result<JsonValue, McpTransportError> {
    loop {
        let now = Instant::now();
        if now >= deadline {
            terminate_child(child);
            return Err(McpTransportError::timeout(timeout));
        }
        let remaining = deadline.saturating_duration_since(now);
        match rx.recv_timeout(POLL_INTERVAL.min(remaining)) {
            Ok(Ok(message)) => {
                if response_id(&message) != Some(id) {
                    continue;
                }
                return response_result(message);
            }
            Ok(Err(error)) => return Err(error),
            Err(RecvTimeoutError::Timeout) => {
                if process_exited(child)? {
                    return Err(McpTransportError::failed(
                        "MCP server exited before responding.",
                    ));
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                return Err(McpTransportError::failed(
                    "MCP server exited before responding.",
                ));
            }
        }
    }
}

fn process_exited(child: &mut Child) -> Result<bool, McpTransportError> {
    child
        .try_wait()
        .map(|status| status.is_some())
        .map_err(|_| McpTransportError::failed("MCP server status check failed."))
}

fn read_stdout_frames(mut stdout: impl Read, tx: Sender<Result<JsonValue, McpTransportError>>) {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        match stdout.read(&mut chunk) {
            Ok(0) => return,
            Ok(read) => {
                buffer.extend_from_slice(&chunk[..read]);
                if buffer.len() > MAX_CLIENT_RESPONSE_BYTES {
                    let _ = tx.send(Err(McpTransportError::failed(
                        "MCP server response exceeded size limit.",
                    )));
                    return;
                }
                match parse_available_messages(&mut buffer) {
                    Ok(messages) => {
                        for message in messages {
                            if tx.send(Ok(message)).is_err() {
                                return;
                            }
                        }
                    }
                    Err(error) => {
                        let _ = tx.send(Err(error));
                        return;
                    }
                }
            }
            Err(_) => {
                let _ = tx.send(Err(McpTransportError::failed(
                    "MCP server stdout read failed.",
                )));
                return;
            }
        }
    }
}

fn drain_stderr(stderr: Option<impl Read + Send + 'static>) {
    if let Some(mut stderr) = stderr {
        thread::spawn(move || {
            let mut sink = [0_u8; 8192];
            let mut read_total = 0_usize;
            while read_total < MAX_CLIENT_RESPONSE_BYTES {
                match stderr.read(&mut sink) {
                    Ok(0) | Err(_) => return,
                    Ok(read) => read_total = read_total.saturating_add(read),
                }
            }
        });
    }
}

fn parse_available_messages(buffer: &mut Vec<u8>) -> Result<Vec<JsonValue>, McpTransportError> {
    let mut messages = Vec::new();
    while let Some(header_end) = find_header_end(buffer) {
        let header = std::str::from_utf8(&buffer[..header_end])
            .map_err(|_| McpTransportError::failed("MCP server sent an invalid header."))?;
        let Some(content_length) = content_length(header) else {
            return Err(McpTransportError::failed(
                "MCP server sent a response without Content-Length.",
            ));
        };
        if content_length > MAX_CLIENT_RESPONSE_BYTES {
            return Err(McpTransportError::failed(
                "MCP server response exceeded size limit.",
            ));
        }
        let body_start = header_end + 4;
        let body_end = body_start.saturating_add(content_length);
        if buffer.len() < body_end {
            break;
        }
        let body = buffer[body_start..body_end].to_vec();
        buffer.drain(..body_end);
        let message = serde_json::from_slice::<JsonValue>(&body)
            .map_err(|_| McpTransportError::failed("MCP server sent invalid JSON."))?;
        messages.push(message);
    }
    Ok(messages)
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn content_length(header: &str) -> Option<usize> {
    header.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if !name.trim().eq_ignore_ascii_case("Content-Length") {
            return None;
        }
        value.trim().parse::<usize>().ok()
    })
}

fn response_id(message: &JsonValue) -> Option<i64> {
    let JsonValue::Object(record) = message else {
        return None;
    };
    match record.get("id") {
        Some(JsonValue::Number(JsonNumber::I64(value))) => Some(*value),
        Some(JsonValue::Number(JsonNumber::U64(value))) => i64::try_from(*value).ok(),
        _ => None,
    }
}

fn response_result(message: JsonValue) -> Result<JsonValue, McpTransportError> {
    let JsonValue::Object(mut record) = message else {
        return Err(McpTransportError::failed(
            "MCP server response was invalid.",
        ));
    };
    if let Some(JsonValue::Object(error)) = record.remove("error") {
        let code = error_code(&error);
        return Err(McpTransportError::tool_error(
            code,
            "MCP server returned error.",
        ));
    }
    Ok(record.remove("result").unwrap_or(JsonValue::Null))
}

fn error_code(error: &JsonObject) -> i64 {
    match error.get("code") {
        Some(JsonValue::Number(JsonNumber::I64(value))) => *value,
        Some(JsonValue::Number(JsonNumber::U64(value))) => i64::try_from(*value).unwrap_or(0),
        _ => 0,
    }
}

fn initialize_request(id: i64) -> JsonValue {
    json_rpc_request(
        id,
        "initialize",
        [
            (
                "protocolVersion".to_owned(),
                JsonValue::String(PROTOCOL_VERSION.to_owned()),
            ),
            (
                "capabilities".to_owned(),
                JsonValue::Object(JsonObject::new()),
            ),
            (
                "clientInfo".to_owned(),
                JsonValue::Object(
                    [
                        ("name".to_owned(), JsonValue::String("runx".to_owned())),
                        ("version".to_owned(), JsonValue::String("0.0.0".to_owned())),
                    ]
                    .into(),
                ),
            ),
        ]
        .into(),
    )
}

fn initialized_notification() -> JsonValue {
    JsonValue::Object(
        [
            ("jsonrpc".to_owned(), JsonValue::String("2.0".to_owned())),
            (
                "method".to_owned(),
                JsonValue::String("notifications/initialized".to_owned()),
            ),
            ("params".to_owned(), JsonValue::Object(JsonObject::new())),
        ]
        .into(),
    )
}

fn tool_call_request(id: i64, tool: &str, arguments: &JsonObject) -> JsonValue {
    json_rpc_request(
        id,
        "tools/call",
        [
            ("name".to_owned(), JsonValue::String(tool.to_owned())),
            ("arguments".to_owned(), JsonValue::Object(arguments.clone())),
        ]
        .into(),
    )
}

fn json_rpc_request(id: i64, method: &str, params: JsonObject) -> JsonValue {
    JsonValue::Object(
        [
            ("jsonrpc".to_owned(), JsonValue::String("2.0".to_owned())),
            ("id".to_owned(), JsonValue::Number(JsonNumber::I64(id))),
            ("method".to_owned(), JsonValue::String(method.to_owned())),
            ("params".to_owned(), JsonValue::Object(params)),
        ]
        .into(),
    )
}

fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
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

fn env_value(env: &BTreeMap<String, String>, name: Option<&JsonValue>) -> String {
    env.get(&js_string(name)).cloned().unwrap_or_default()
}

fn js_string(value: Option<&JsonValue>) -> String {
    match value {
        None | Some(JsonValue::Null) => String::new(),
        Some(JsonValue::String(value)) => value.clone(),
        Some(JsonValue::Bool(value)) => value.to_string(),
        Some(JsonValue::Number(value)) => json_number_string(value),
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| js_string(Some(value)))
            .collect::<Vec<_>>()
            .join(","),
        Some(JsonValue::Object(_)) => "[object Object]".to_owned(),
    }
}

fn json_number_string(value: &JsonNumber) -> String {
    match value {
        JsonNumber::I64(value) => value.to_string(),
        JsonNumber::U64(value) => value.to_string(),
        JsonNumber::F64(value) if value.fract() == 0.0 => format!("{value:.0}"),
        JsonNumber::F64(value) => value.to_string(),
    }
}

fn timeout_from_source(timeout_seconds: Option<u64>) -> Duration {
    let timeout_ms = timeout_seconds
        .map(|seconds| seconds.saturating_mul(1000))
        .unwrap_or(DEFAULT_TIMEOUT_MS)
        .max(MIN_TIMEOUT_MS);
    Duration::from_millis(timeout_ms)
}

fn metadata_for(
    source: &runx_parser::SkillSource,
    sandbox: Option<JsonObject>,
) -> Result<JsonObject, RuntimeError> {
    let mut mcp = JsonObject::new();
    mcp.insert(
        "tool".to_owned(),
        JsonValue::String(source.tool.clone().unwrap_or_default()),
    );
    let server = source.server.as_ref();
    mcp.insert(
        "server_command_hash".to_owned(),
        JsonValue::String(sha256_hex(
            server
                .map(|server| server.command.as_bytes())
                .unwrap_or(b""),
        )),
    );
    let args = serde_json::to_string(&server.map(|server| &server.args))
        .map_err(|source| RuntimeError::json("serializing MCP server args", source))?;
    mcp.insert(
        "server_args_hash".to_owned(),
        JsonValue::String(sha256_hex(args.as_bytes())),
    );

    let mut metadata = JsonObject::new();
    metadata.insert("mcp".to_owned(), JsonValue::Object(mcp));
    if let Some(sandbox) = sandbox.filter(|sandbox| !sandbox.is_empty()) {
        metadata.insert("sandbox".to_owned(), JsonValue::Object(sandbox));
    }
    Ok(metadata)
}

fn failure(message: impl Into<String>, started: Instant, metadata: JsonObject) -> SkillOutput {
    let message = message.into();
    SkillOutput {
        status: InvocationStatus::Failure,
        stdout: String::new(),
        stderr: message,
        exit_code: None,
        duration_ms: duration_ms(started),
        metadata,
    }
}

fn duration_ms(started: Instant) -> u64 {
    let millis = started.elapsed().as_millis();
    u64::try_from(millis).unwrap_or(u64::MAX)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}
