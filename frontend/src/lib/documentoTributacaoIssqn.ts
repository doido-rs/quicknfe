/** Alinhado à NFS-e Nacional (`tribISSQN`). */
export const TRIBUTACAO_ISSQN_OPTIONS = [
  { value: '1', label: 'Operação tributável' },
  { value: '2', label: 'Imunidade' },
  { value: '3', label: 'Exportação de serviço' },
  { value: '4', label: 'Não incidência' },
] as const

/** Alinhado à NFS-e Nacional (`regEspTrib`). */
export const REGIME_ESPECIAL_TRIB_ISSQN_OPTIONS = [
  { value: '0', label: 'Nenhum' },
  { value: '1', label: 'Microempresa municipal' },
  { value: '2', label: 'Estimativa' },
  { value: '3', label: 'Sociedade de profissionais' },
  { value: '4', label: 'Cooperativa' },
  { value: '5', label: 'Microempresário individual (MEI)' },
  { value: '6', label: 'Microempresa ou empresa de pequeno porte (ME/EPP)' },
] as const

export function tributacaoIssqnLabel(value: unknown): string {
  if (value == null || value === '') return '—'
  const found = TRIBUTACAO_ISSQN_OPTIONS.find((o) => o.value === String(value))
  return found?.label ?? String(value)
}

export function regimeEspecialTribIssqnLabel(value: unknown): string {
  if (value == null || value === '') return '—'
  const found = REGIME_ESPECIAL_TRIB_ISSQN_OPTIONS.find((o) => o.value === String(value))
  return found?.label ?? String(value)
}

export function simNaoLabel(value: unknown): string {
  if (value === true || value === 'true' || value === 1 || value === '1') return 'Sim'
  if (value === false || value === 'false' || value === 0 || value === '0') return 'Não'
  return '—'
}
