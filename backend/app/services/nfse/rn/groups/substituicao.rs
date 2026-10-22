//! Group `Substituicao` — generated per-cod entrypoints.
use crate::services::nfse::dps_context::DpsContext;
use crate::services::nfse::rn::violations::RnViolation;

pub fn e0042(ctx: &DpsContext) -> Result<(), RnViolation> {
    super::common::evaluate_rule("E0042", ctx)
}
pub fn e0078(ctx: &DpsContext) -> Result<(), RnViolation> {
    super::common::evaluate_rule("E0078", ctx)
}
