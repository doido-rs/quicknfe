/** Campos ISSQN omitidos para ME/EPP + apuração SN sem retenção (E0625). */
export const ISSQN_ALIQUOTA_FIELD_KEYS = [
  'issqn_aliquota',
  'issqn_base_calculo',
  'issqn_valor',
] as const

const ME_EPP_REGIMES = new Set([
  'SIMPLES_NACIONAL',
  'SIMPLES_NACIONAL_EXCESSO_SUBLIMITE',
])

/** ME/EPP com ISSQN apurado pelo SN (`regApTribSN = 1` no XML). */
export function prestadorMeEppApuraIssqnSn(regimeTributario: string | undefined): boolean {
  if (!regimeTributario?.trim()) return false
  return ME_EPP_REGIMES.has(regimeTributario.trim())
}

export function omitirCamposAliquotaIssqnMeEpp(
  regimeTributario: string | undefined,
  issqnRetencaoForm: string | undefined,
  issqnBeneficioMunicipalForm: string | undefined,
): boolean {
  if (!prestadorMeEppApuraIssqnSn(regimeTributario)) return false
  if (issqnRetencaoForm === 'true') return false
  if (issqnBeneficioMunicipalForm === 'true') return false
  return true
}
