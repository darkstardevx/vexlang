//! ## Vex [Vexlang] ##
//! # Version: 0.0.1 (SemVer)
//! # Inspired by Rust & Zig
//! # Built for Cybercore ecosystem code consistency and ease
//! # GitHub: [Vex] (https://github.com/darkstardevx/vex)
//! # Tag reference doc in /home/raven/devspace/docs/tags/TAG_API.md
//!

use crate::ir::IrProgram;

/// A backend consumes only verified IR. No backend is shipped in this phase.
pub trait Backend {
    type Error: std::error::Error;

    fn name(&self) -> &'static str;
    fn emit(&self, program: &IrProgram) -> Result<String, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendError {
    pub code: &'static str,
    pub backend: &'static str,
    pub message: String,
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]: {}", self.code, self.backend, self.message)
    }
}

impl std::error::Error for BackendError {}

/// QBE remains an explicit boundary, rather than a partial or fake emitter.
pub struct QbeBackend;

impl Backend for QbeBackend {
    type Error = BackendError;

    fn name(&self) -> &'static str {
        "qbe"
    }

    fn emit(&self, _program: &IrProgram) -> Result<String, Self::Error> {
        Err(BackendError {
            code: "BE001",
            backend: self.name(),
            message: "native code generation is not implemented".into(),
        })
    }
}
