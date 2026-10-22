/** CST PIS/COFINS (NFS-e Nacional / SPED). */
export const CST_PIS_COFINS_OPTIONS = [
  { value: '00', label: '00 — Nenhum' },
  { value: '01', label: '01 — Alíquota básica' },
  { value: '02', label: '02 — Alíquota diferenciada' },
  { value: '03', label: '03 — Alíquota por unidade de medida' },
  { value: '04', label: '04 — Monofásica, revenda a alíquota zero' },
  { value: '05', label: '05 — Substituição tributária' },
  { value: '06', label: '06 — Alíquota zero' },
  { value: '07', label: '07 — Isenta' },
  { value: '08', label: '08 — Sem incidência' },
  { value: '09', label: '09 — Com suspensão' },
  { value: '49', label: '49 — Outras operações de saída' },
  { value: '99', label: '99 — Outras operações' },
] as const

/** Tipo de retenção PIS/COFINS/CSLL (`tpRetPisCofins`, NT 007). */
export const TP_RET_PIS_COFINS_CSLL_OPTIONS = [
  { value: '0', label: '0 — PIS/COFINS/CSLL não retidos' },
  { value: '3', label: '3 — PIS/COFINS/CSLL retidos' },
  { value: '4', label: '4 — PIS/COFINS retidos; CSLL não retido' },
  { value: '5', label: '5 — PIS retido; COFINS/CSLL não retidos' },
  { value: '6', label: '6 — COFINS retido; PIS/CSLL não retidos' },
  { value: '7', label: '7 — COFINS/CSLL retidos; PIS não retido' },
  { value: '8', label: '8 — PIS/CSLL retidos; COFINS não retido' },
  { value: '9', label: '9 — PIS/COFINS não retidos; CSLL retido' },
] as const

const CST_PIS_COFINS_WIRE_TO_FORM: Record<string, string> = {
  NENHUM: '00',
  ALIQUOTA_BASICA: '01',
  ALIQUOTA_DIFERENCIADA: '02',
  ALIQUOTA_UNIDADE: '03',
  MONOFASICA_REVENDA_ZERO: '04',
  SUBSTITUICAO_TRIBUTARIA: '05',
  ALIQUOTA_ZERO: '06',
  ISENTA: '07',
  SEM_INCIDENCIA: '08',
  COM_SUSPENSAO: '09',
  OUTRAS_SAIDA_49: '49',
  OUTRAS_99: '99',
}

const TP_RET_PIS_COFINS_WIRE_TO_FORM: Record<string, string> = {
  NAO_RETIDOS: '0',
  RETIDOS: '3',
  PIS_COFINS_RETIDOS_CSLL_NAO: '4',
  PIS_RETIDO: '5',
  COFINS_RETIDO: '6',
  COFINS_CSLL_RETIDOS_PIS_NAO: '7',
  PIS_CSLL_RETIDOS_COFINS_NAO: '8',
  PIS_COFINS_NAO_RETIDOS_CSLL_RETIDO: '9',
}

export function cstPisCofinsToFormValue(value: unknown): string {
  if (value == null || value === '') return ''
  const s = String(value).trim()
  const padded = /^\d$/.test(s) ? `0${s}` : s
  if (CST_PIS_COFINS_OPTIONS.some((o) => o.value === padded)) return padded
  const upper = s.toUpperCase()
  return CST_PIS_COFINS_WIRE_TO_FORM[upper] ?? padded
}

export function tpRetPisCofinsCsllToFormValue(value: unknown): string {
  if (value == null || value === '') return ''
  const s = String(value).trim()
  if (TP_RET_PIS_COFINS_CSLL_OPTIONS.some((o) => o.value === s)) return s
  const upper = s.toUpperCase()
  return TP_RET_PIS_COFINS_WIRE_TO_FORM[upper] ?? s
}

export function cstPisCofinsLabel(value: unknown): string {
  if (value == null || value === '') return '—'
  const key = cstPisCofinsToFormValue(value)
  const found = CST_PIS_COFINS_OPTIONS.find((o) => o.value === key)
  return found?.label ?? String(value)
}

export function tpRetPisCofinsCsllLabel(value: unknown): string {
  if (value == null || value === '') return '—'
  const key = tpRetPisCofinsCsllToFormValue(value)
  const found = TP_RET_PIS_COFINS_CSLL_OPTIONS.find((o) => o.value === key)
  return found?.label ?? String(value)
}
