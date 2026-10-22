//! Group `CadastroParte` — generated per-cod entrypoints.
use crate::services::nfse::dps_context::DpsContext;
use crate::services::nfse::rn::violations::RnViolation;

pub fn e1280(ctx: &DpsContext) -> Result<(), RnViolation> {
    super::common::evaluate_rule("E1280", ctx)
}
pub fn e1284(ctx: &DpsContext) -> Result<(), RnViolation> {
    super::common::evaluate_rule("E1284", ctx)
}
pub fn e1286(ctx: &DpsContext) -> Result<(), RnViolation> {
    super::common::evaluate_rule("E1286", ctx)
}
