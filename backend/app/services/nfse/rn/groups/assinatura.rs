//! Group `Assinatura` — generated per-cod entrypoints.
use crate::services::nfse::dps_context::DpsContext;
use crate::services::nfse::rn::violations::RnViolation;

pub fn e1630(ctx: &DpsContext) -> Result<(), RnViolation> {
    super::common::evaluate_rule("E1630", ctx)
}
