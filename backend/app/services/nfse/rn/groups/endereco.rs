//! Group `Endereco` — generated per-cod entrypoints.
use crate::services::nfse::dps_context::DpsContext;
use crate::services::nfse::rn::violations::RnViolation;

pub fn e0380(ctx: &DpsContext) -> Result<(), RnViolation> {
    super::common::evaluate_rule("E0380", ctx)
}
pub fn e0384(ctx: &DpsContext) -> Result<(), RnViolation> {
    super::common::evaluate_rule("E0384", ctx)
}
