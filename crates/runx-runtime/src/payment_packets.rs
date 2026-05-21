// rust-style-allow: long-function because payment packet parsing accepts the
// current graph output envelopes while payment execution is being generalized
// across mock and provider-backed rails.
use runx_contracts::{JsonNumber, JsonObject, JsonValue};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentReservationPacket {
    pub amount_minor: u64,
    pub currency: String,
    pub rail: String,
    pub counterparty: String,
    pub operation: String,
    pub idempotency_key: String,
    pub spend_capability_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentRailPacket {
    pub result: Option<PaymentRailResult>,
    pub proof: Option<PaymentRailProof>,
    pub recovery_status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentRailResult {
    pub status: Option<String>,
    pub rail: Option<String>,
    pub amount_minor: Option<u64>,
    pub currency: Option<String>,
    pub counterparty: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentRailProof {
    pub proof_ref: String,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentRefusalPacket {
    pub reason_code: String,
    pub rail_call_performed: bool,
    pub ledger_spend_recorded: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaidToolPaymentPacket {
    pub payment_proof_ref: String,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum PaymentPacketError {
    #[error("payment packet is missing {field}")]
    MissingField { field: &'static str },
    #[error("payment packet field {field} has an invalid value")]
    InvalidField { field: &'static str },
}

// rust-style-allow: long-function because reservation packets may derive fields
// from either the authority envelope or the spend-capability binding while the
// payment execution boundary is still being factored.
pub fn read_payment_reservation_packet(
    outputs: &JsonObject,
) -> Result<Option<PaymentReservationPacket>, PaymentPacketError> {
    let Some(data) = packet_data(outputs, "payment_reservation_packet") else {
        return Ok(None);
    };
    let Some(binding) = object_path(
        data,
        &["reserved_payment_authority", "spend_capability_binding"],
    )
    .or_else(|| object_path(data, &["spend_capability_binding"])) else {
        return Ok(None);
    };
    let payment_bounds = object_path(
        data,
        &[
            "reserved_payment_authority",
            "child_authority",
            "bounds",
            "payment",
        ],
    )
    .or_else(|| {
        object_path(
            data,
            &[
                "reserved_payment_authority",
                "parent_authority",
                "bounds",
                "payment",
            ],
        )
    });

    Ok(Some(PaymentReservationPacket {
        amount_minor: required_u64(
            binding,
            "amount_minor",
            "spend_capability_binding.amount_minor",
        )?,
        currency: required_string(binding, "currency", "spend_capability_binding.currency")?
            .to_owned(),
        rail: required_string(binding, "rail", "spend_capability_binding.rail")?.to_owned(),
        counterparty: required_string(
            binding,
            "counterparty",
            "spend_capability_binding.counterparty",
        )?
        .to_owned(),
        operation: payment_bounds
            .and_then(|bounds| string_field(bounds, "operation"))
            .ok_or(PaymentPacketError::MissingField {
                field: "reserved_payment_authority.*.bounds.payment.operation",
            })?
            .to_owned(),
        idempotency_key: required_string(
            binding,
            "idempotency_key",
            "spend_capability_binding.idempotency_key",
        )?
        .to_owned(),
        spend_capability_ref: object_path(data, &["spend_capability_ref"])
            .and_then(|reference| string_field(reference, "uri"))
            .ok_or(PaymentPacketError::MissingField {
                field: "spend_capability_ref.uri",
            })?
            .to_owned(),
    }))
}

pub fn read_payment_rail_packet(
    outputs: &JsonObject,
) -> Result<Option<PaymentRailPacket>, PaymentPacketError> {
    let Some(data) = packet_data(outputs, "payment_rail_packet") else {
        return Ok(None);
    };
    let result = object_path(data, &["rail_result"])
        .map(|result| {
            Ok(PaymentRailResult {
                status: string_field(result, "status").map(str::to_owned),
                rail: string_field(result, "rail").map(str::to_owned),
                amount_minor: optional_u64(result, "amount_minor", "rail_result.amount_minor")?,
                currency: string_field(result, "currency").map(str::to_owned),
                counterparty: string_field(result, "counterparty").map(str::to_owned),
            })
        })
        .transpose()?;
    let proof = object_path(data, &["rail_proof"])
        .map(|proof| {
            Ok(PaymentRailProof {
                proof_ref: required_string(proof, "proof_ref", "rail_proof.proof_ref")?.to_owned(),
                idempotency_key: required_string(
                    proof,
                    "idempotency_key",
                    "rail_proof.idempotency_key",
                )?
                .to_owned(),
            })
        })
        .transpose()?;

    Ok(Some(PaymentRailPacket {
        result,
        proof,
        recovery_status: object_path(data, &["recovery_hint"])
            .and_then(|hint| string_field(hint, "status"))
            .map(str::to_owned),
    }))
}

pub fn read_payment_refusal_packet(
    outputs: &JsonObject,
) -> Result<Option<PaymentRefusalPacket>, PaymentPacketError> {
    let refusal = packet_data(outputs, "payment_refusal_packet").or_else(|| {
        packet_data(outputs, "payment_reservation_packet")
            .and_then(|data| object_path(data, &["payment_refusal_packet"]))
    });
    let Some(refusal) = refusal else {
        return Ok(None);
    };
    Ok(Some(PaymentRefusalPacket {
        reason_code: required_string(refusal, "reason_code", "payment_refusal_packet.reason_code")?
            .to_owned(),
        rail_call_performed: bool_field(refusal, "rail_call_performed").unwrap_or(false),
        ledger_spend_recorded: bool_field(refusal, "ledger_spend_recorded").unwrap_or(false),
    }))
}

pub fn read_paid_tool_packet(
    outputs: &JsonObject,
) -> Result<Option<PaidToolPaymentPacket>, PaymentPacketError> {
    let Some(result) = object_path(outputs, &["paid_echo_result"]) else {
        return Ok(None);
    };
    Ok(Some(PaidToolPaymentPacket {
        payment_proof_ref: required_string(
            result,
            "payment_proof_ref",
            "paid_echo_result.payment_proof_ref",
        )?
        .to_owned(),
    }))
}

fn packet_data<'a>(outputs: &'a JsonObject, packet: &str) -> Option<&'a JsonObject> {
    object_path(outputs, &[packet, "data"])
}

fn object_path<'a>(object: &'a JsonObject, path: &[&str]) -> Option<&'a JsonObject> {
    let mut current = object;
    for (index, segment) in path.iter().enumerate() {
        let value = current.get(*segment)?;
        if index + 1 == path.len() {
            return match value {
                JsonValue::Object(object) => Some(object),
                _ => None,
            };
        }
        let JsonValue::Object(next) = value else {
            return None;
        };
        current = next;
    }
    Some(current)
}

fn required_string<'a>(
    object: &'a JsonObject,
    key: &'static str,
    field: &'static str,
) -> Result<&'a str, PaymentPacketError> {
    string_field(object, key).ok_or(PaymentPacketError::MissingField { field })
}

fn string_field<'a>(object: &'a JsonObject, key: &str) -> Option<&'a str> {
    match object.get(key)? {
        JsonValue::String(value) if !value.is_empty() => Some(value),
        _ => None,
    }
}

fn bool_field(object: &JsonObject, key: &str) -> Option<bool> {
    match object.get(key)? {
        JsonValue::Bool(value) => Some(*value),
        _ => None,
    }
}

fn required_u64(
    object: &JsonObject,
    key: &'static str,
    field: &'static str,
) -> Result<u64, PaymentPacketError> {
    match u64_field(object, key) {
        Some(value) => Ok(value),
        None if object.contains_key(key) => Err(PaymentPacketError::InvalidField { field }),
        None => Err(PaymentPacketError::MissingField { field }),
    }
}

fn optional_u64(
    object: &JsonObject,
    key: &'static str,
    field: &'static str,
) -> Result<Option<u64>, PaymentPacketError> {
    match u64_field(object, key) {
        Some(value) => Ok(Some(value)),
        None if object.contains_key(key) => Err(PaymentPacketError::InvalidField { field }),
        None => Ok(None),
    }
}

fn u64_field(object: &JsonObject, key: &'static str) -> Option<u64> {
    match object.get(key)? {
        JsonValue::Number(JsonNumber::U64(value)) => Some(*value),
        JsonValue::Number(JsonNumber::I64(value)) => u64::try_from(*value).ok(),
        JsonValue::Number(JsonNumber::F64(value))
            if value.is_finite() && value.fract() == 0.0 && *value >= 0.0 =>
        {
            Some(*value as u64)
        }
        JsonValue::Number(_) => None,
        _ => None,
    }
}
