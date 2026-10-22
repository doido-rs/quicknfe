//! Group `Evento` — generated per-cod entrypoints.
use crate::services::nfse::dps_context::DpsContext;
use crate::services::nfse::rn::violations::RnViolation;

pub fn e0398(ctx: &DpsContext) -> Result<(), RnViolation> {
    super::common::evaluate_rule("E0398", ctx)
}
pub fn e0402(ctx: &DpsContext) -> Result<(), RnViolation> {
    super::common::evaluate_rule("E0402", ctx)
}
