//! Group `ServicoItem` — generated per-cod entrypoints.
use crate::services::nfse::dps_context::DpsContext;
use crate::services::nfse::rn::violations::RnViolation;

pub fn e0310(ctx: &DpsContext) -> Result<(), RnViolation> {
    super::common::evaluate_rule("E0310", ctx)
}
pub fn e0314(ctx: &DpsContext) -> Result<(), RnViolation> {
    super::common::evaluate_rule("E0314", ctx)
}
