// HANDWRITE-BEGIN gap="missing-generator:logic:40123b56" tracker="pending-tracker" reason="Define and dispatch the versioned newline-framed JSON request, response, result, and recoverable error contract."
use std::collections::BTreeSet;

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    native_agent_pty::PtySize,
    terminal_core::{TerminalCore, TerminalCoreError, TerminalFrame, TerminalProfile},
};

pub const CORE_PROTOCOL_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CoreMethod {
    Hello,
    Launch,
    Poll,
    Input,
    Resize,
    Interrupt,
    Terminate,
    Shutdown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreRequest {
    pub protocol_version: u16,
    pub request_id: u64,
    pub method: CoreMethod,
    #[serde(default)]
    pub params: Value,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreResponse {
    pub protocol_version: u16,
    pub request_id: u64,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<CoreResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CoreError>,
}

impl CoreResponse {
    fn success(request_id: u64, result: CoreResult) -> Self {
        Self {
            protocol_version: CORE_PROTOCOL_VERSION,
            request_id,
            ok: true,
            result: Some(result),
            error: None,
        }
    }

    fn failure(request_id: u64, code: CoreErrorCode, message: impl Into<String>) -> Self {
        Self {
            protocol_version: CORE_PROTOCOL_VERSION,
            request_id,
            ok: false,
            result: None,
            error: Some(CoreError {
                code,
                message: message.into(),
            }),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CoreResult {
    Hello {
        profiles: [TerminalProfile; 4],
        default_shell: String,
    },
    Session {
        frame: TerminalFrame,
    },
    Ack,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreError {
    pub code: CoreErrorCode,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CoreErrorCode {
    InvalidJson,
    UnsupportedVersion,
    DuplicateRequest,
    InvalidParams,
    InvalidTab,
    InvalidWorkingDirectory,
    UnavailableProgram,
    AlreadyRunning,
    MissingSession,
    PtyOperation,
}

#[derive(Debug)]
pub struct DispatchOutcome {
    pub response: CoreResponse,
    pub shutdown: bool,
}

pub struct ProtocolServer {
    core: TerminalCore,
    seen_request_ids: BTreeSet<u64>,
}

impl Default for ProtocolServer {
    fn default() -> Self {
        Self::new(TerminalCore::default())
    }
}

impl ProtocolServer {
    pub fn new(core: TerminalCore) -> Self {
        Self {
            core,
            seen_request_ids: BTreeSet::new(),
        }
    }

    pub fn handle_line(&mut self, line: &str) -> DispatchOutcome {
        let request = match serde_json::from_str::<CoreRequest>(line) {
            Ok(request) => request,
            Err(error) => {
                return DispatchOutcome {
                    response: CoreResponse::failure(
                        request_id_from_invalid_json(line),
                        CoreErrorCode::InvalidJson,
                        format!("invalid request JSON: {error}"),
                    ),
                    shutdown: false,
                };
            }
        };
        if request.protocol_version != CORE_PROTOCOL_VERSION {
            return DispatchOutcome {
                response: CoreResponse::failure(
                    request.request_id,
                    CoreErrorCode::UnsupportedVersion,
                    format!(
                        "unsupported protocol version {}; expected {CORE_PROTOCOL_VERSION}",
                        request.protocol_version
                    ),
                ),
                shutdown: false,
            };
        }
        if request.request_id == 0 || !self.seen_request_ids.insert(request.request_id) {
            return DispatchOutcome {
                response: CoreResponse::failure(
                    request.request_id,
                    CoreErrorCode::DuplicateRequest,
                    "request id must be non-zero and unique for this sidecar process",
                ),
                shutdown: false,
            };
        }

        let request_id = request.request_id;
        let shutdown = request.method == CoreMethod::Shutdown;
        let result = self.dispatch(request);
        DispatchOutcome {
            response: match result {
                Ok(result) => CoreResponse::success(request_id, result),
                Err((code, message)) => CoreResponse::failure(request_id, code, message),
            },
            shutdown,
        }
    }

    pub fn terminate_all(&mut self) {
        self.core.terminate_all();
    }

    fn dispatch(&mut self, request: CoreRequest) -> Result<CoreResult, (CoreErrorCode, String)> {
        match request.method {
            CoreMethod::Hello => Ok(CoreResult::Hello {
                profiles: TerminalProfile::DEFAULTS,
                default_shell: self.core.default_shell().display().to_string(),
            }),
            CoreMethod::Launch => {
                let params: LaunchParams = decode_params(request.params)?;
                let frame = self
                    .core
                    .launch(
                        &params.tab_id,
                        params.profile,
                        params.cwd,
                        PtySize {
                            rows: params.rows.unwrap_or(28),
                            cols: params.cols.unwrap_or(100),
                            pixel_width: 0,
                            pixel_height: 0,
                        },
                    )
                    .map_err(map_terminal_error)?;
                Ok(CoreResult::Session { frame })
            }
            CoreMethod::Poll => {
                let params: TabParams = decode_params(request.params)?;
                let frame = self.core.poll(&params.tab_id).map_err(map_terminal_error)?;
                Ok(CoreResult::Session { frame })
            }
            CoreMethod::Input => {
                let params: InputParams = decode_params(request.params)?;
                let bytes = BASE64_STANDARD
                    .decode(params.data_base64)
                    .map_err(|error| {
                        (
                            CoreErrorCode::InvalidParams,
                            format!("input dataBase64 is invalid: {error}"),
                        )
                    })?;
                let frame = self
                    .core
                    .input(&params.tab_id, &bytes)
                    .map_err(map_terminal_error)?;
                Ok(CoreResult::Session { frame })
            }
            CoreMethod::Resize => {
                let params: ResizeParams = decode_params(request.params)?;
                let frame = self
                    .core
                    .resize(&params.tab_id, params.rows, params.cols)
                    .map_err(map_terminal_error)?;
                Ok(CoreResult::Session { frame })
            }
            CoreMethod::Interrupt => {
                let params: TabParams = decode_params(request.params)?;
                let frame = self
                    .core
                    .interrupt(&params.tab_id)
                    .map_err(map_terminal_error)?;
                Ok(CoreResult::Session { frame })
            }
            CoreMethod::Terminate => {
                let params: TabParams = decode_params(request.params)?;
                let frame = self
                    .core
                    .terminate(&params.tab_id)
                    .map_err(map_terminal_error)?;
                Ok(CoreResult::Session { frame })
            }
            CoreMethod::Shutdown => {
                self.core.terminate_all();
                Ok(CoreResult::Ack)
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LaunchParams {
    tab_id: String,
    profile: TerminalProfile,
    cwd: String,
    rows: Option<u16>,
    cols: Option<u16>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TabParams {
    tab_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InputParams {
    tab_id: String,
    data_base64: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResizeParams {
    tab_id: String,
    rows: u16,
    cols: u16,
}

fn decode_params<T: for<'de> Deserialize<'de>>(
    params: Value,
) -> Result<T, (CoreErrorCode, String)> {
    serde_json::from_value(params).map_err(|error| {
        (
            CoreErrorCode::InvalidParams,
            format!("invalid request params: {error}"),
        )
    })
}

fn request_id_from_invalid_json(line: &str) -> u64 {
    serde_json::from_str::<Value>(line)
        .ok()
        .and_then(|value| value.get("requestId").and_then(Value::as_u64))
        .unwrap_or(0)
}

fn map_terminal_error(error: TerminalCoreError) -> (CoreErrorCode, String) {
    let code = match &error {
        TerminalCoreError::InvalidTabId => CoreErrorCode::InvalidTab,
        TerminalCoreError::InvalidWorkingDirectory(_) => CoreErrorCode::InvalidWorkingDirectory,
        TerminalCoreError::UnavailableProgram(_) => CoreErrorCode::UnavailableProgram,
        TerminalCoreError::AlreadyRunning(_) => CoreErrorCode::AlreadyRunning,
        TerminalCoreError::MissingSession(_) => CoreErrorCode::MissingSession,
        TerminalCoreError::Operation(_) => CoreErrorCode::PtyOperation,
    };
    (code, error.to_string())
}

// HANDWRITE-END
