//! Group `Plataforma` — generated per-cod entrypoints.
use crate::services::nfse::dps_context::DpsContext;
use crate::services::nfse::rn::violations::RnViolation;

pub fn e1260(ctx: &DpsContext) -> Result<(), RnViolation> {
    super::common::evaluate_rule("E1260", ctx)
}
