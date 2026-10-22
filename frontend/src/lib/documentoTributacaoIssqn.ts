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

/** Wire JSON (`OPERACAO_TRIBUTAVEL`) → valor do select (`1`). */
const TRIB_ISSQN_WIRE_TO_FORM: Record<string, string> = {
  OPERACAO_TRIBUTAVEL: '1',
  IMUNIDADE: '2',
  EXPORTACAO_SERVICO: '3',
  NAO_INCIDENCIA: '4',
}

const REG_ESP_TRIB_WIRE_TO_FORM: Record<string, string> = {
  NENHUM: '0',
  MICROEMPRESA_MUNICIPAL: '1',
  ESTIMATIVA: '2',
  SOCIEDADE_PROFISSIONAIS: '3',
  COOPERATIVA: '4',
  MEI: '5',
  ME_EPP: '6',
}

export function tribIssqnToFormValue(value: unknown): string {
  if (value == null || value === '') return ''
  const s = String(value).trim()
  if (TRIBUTACAO_ISSQN_OPTIONS.some((o) => o.value === s)) return s
  const upper = s.toUpperCase()
  return TRIB_ISSQN_WIRE_TO_FORM[upper] ?? s
}

export function regEspTribIssqnToFormValue(value: unknown): string {
  if (value == null || value === '') return ''
  const s = String(value).trim()
  if (REGIME_ESPECIAL_TRIB_ISSQN_OPTIONS.some((o) => o.value === s)) return s
  const upper = s.toUpperCase()
  return REG_ESP_TRIB_WIRE_TO_FORM[upper] ?? s
}

export function tributacaoIssqnLabel(value: unknown): string {
  if (value == null || value === '') return '—'
  const formVal = tribIssqnToFormValue(value)
  const found = TRIBUTACAO_ISSQN_OPTIONS.find((o) => o.value === formVal)
  return found?.label ?? String(value)
}

export function regimeEspecialTribIssqnLabel(value: unknown): string {
  if (value == null || value === '') return '—'
  const formVal = regEspTribIssqnToFormValue(value)
  const found = REGIME_ESPECIAL_TRIB_ISSQN_OPTIONS.find((o) => o.value === formVal)
  return found?.label ?? String(value)
}

export function simNaoLabel(value: unknown): string {
  if (value === true || value === 'true' || value === 1 || value === '1') return 'Sim'
  if (value === false || value === 'false' || value === 0 || value === '0') return 'Não'
  return '—'
}
