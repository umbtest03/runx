use std::io::{self, Write};
use std::process::ExitCode;

use runx_contracts::{JsonObject, JsonValue};

pub(super) fn write_skill_output(value: &JsonValue, json: bool, exit_code: ExitCode) -> ExitCode {
    if !json {
        return write_text_with_exit(value, exit_code);
    }
    write_json_with_exit(value, exit_code)
}

pub(super) fn skill_result_exit_code(value: &JsonValue) -> ExitCode {
    match value {
        JsonValue::Object(object) => match object.get("status") {
            Some(JsonValue::String(status)) if status == "needs_agent" => ExitCode::from(2),
            _ => ExitCode::SUCCESS,
        },
        _ => ExitCode::SUCCESS,
    }
}

fn write_json_with_exit(value: &JsonValue, exit_code: ExitCode) -> ExitCode {
    match serde_json::to_string_pretty(value) {
        Ok(json) => {
            let mut stdout = io::stdout().lock();
            let result = stdout
                .write_all(json.as_bytes())
                .and_then(|_| stdout.write_all(b"\n"));
            match result {
                Ok(()) => exit_code,
                Err(_) => ExitCode::from(1),
            }
        }
        Err(error) => {
            let _ignored = writeln!(
                io::stderr(),
                "runx: failed to serialize skill result: {error}"
            );
            ExitCode::from(1)
        }
    }
}

fn write_text_with_exit(value: &JsonValue, exit_code: ExitCode) -> ExitCode {
    let mut stdout = io::stdout().lock();
    let result = write_skill_text(&mut stdout, value);
    match result {
        Ok(()) => exit_code,
        Err(_) => ExitCode::from(1),
    }
}

fn write_skill_text(writer: &mut dyn Write, value: &JsonValue) -> io::Result<()> {
    let Some(object) = value.as_object() else {
        let text = serde_json::to_string(value).unwrap_or_else(|_| "null".to_owned());
        return writeln!(writer, "{text}");
    };
    writeln!(
        writer,
        "status: {}",
        object_string(object, "status").unwrap_or("unknown")
    )?;
    if let Some(skill_name) = object_string(object, "skill_name") {
        writeln!(writer, "skill: {skill_name}")?;
    }
    if let Some(run_id) = object_string(object, "run_id") {
        writeln!(writer, "run_id: {run_id}")?;
    }
    if let Some(receipt_id) = object_string(object, "receipt_id") {
        writeln!(writer, "receipt_id: {receipt_id}")?;
    }
    if let Some(summary) = object
        .get("closure")
        .and_then(JsonValue::as_object)
        .and_then(|closure| object_string(closure, "summary"))
    {
        writeln!(writer, "summary: {summary}")?;
    } else if let Some(summary) = summary_from_payload(object) {
        writeln!(writer, "summary: {summary}")?;
    }
    if let Some(requests) = object.get("requests").and_then(JsonValue::as_array) {
        writeln!(writer, "pending_requests: {}", requests.len())?;
        for request in requests {
            if let Some(request) = request.as_object() {
                let id = object_string(request, "id").unwrap_or("<unknown>");
                let kind = object_string(request, "kind").unwrap_or("<unknown>");
                writeln!(writer, "- {kind}: {id}")?;
            }
        }
        if let Some(run_id) = object_string(object, "run_id") {
            writeln!(
                writer,
                "next: resolve the request, then rerun with --run-id {run_id} --answers <answers.json>"
            )?;
        }
    }
    Ok(())
}

fn summary_from_payload(object: &JsonObject) -> Option<&str> {
    object
        .get("payload")
        .and_then(JsonValue::as_object)
        .and_then(summary_from_object)
        .or_else(|| {
            object
                .get("execution")
                .and_then(JsonValue::as_object)
                .and_then(|execution| execution.get("structured_output"))
                .and_then(JsonValue::as_object)
                .and_then(summary_from_object)
        })
}

fn summary_from_object(object: &JsonObject) -> Option<&str> {
    object_string(object, "summary").or_else(|| {
        object
            .get("forecast_packet")
            .and_then(JsonValue::as_object)
            .and_then(|packet| object_string(packet, "summary"))
    })
}

fn object_string<'a>(object: &'a JsonObject, key: &str) -> Option<&'a str> {
    object.get(key).and_then(JsonValue::as_str)
}
