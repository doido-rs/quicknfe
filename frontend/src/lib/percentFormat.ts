/**
 * Campos de percentual (%) — tipo `percent` em `definitions.ts`.
 * Estado do formulário: string canônica com ponto e até 4 decimais (`"5.2500"`).
 */
export const PERCENT_DECIMALS = 4

export const PERCENT_FIELD_KEYS = ['issqn_aliquota', 'aliquota', 'aliquota_ibs', 'aliquota_cbs'] as const

export type PercentFieldKey = (typeof PERCENT_FIELD_KEYS)[number]

export function isPercentFieldKey(key: string): key is PercentFieldKey {
  return (PERCENT_FIELD_KEYS as readonly string[]).includes(key)
}

export function roundPercent(n: number): number {
  const factor = 10 ** PERCENT_DECIMALS
  return Math.round(n * factor) / factor
}

/** API / número → valor canônico do form. */
export function normalizePercentToForm(value: unknown): string {
  if (value == null || value === '') return ''
  if (typeof value === 'string' && value.trim() === '') return ''
  const n =
    typeof value === 'number'
      ? value
      : parsePercentFromDisplay(String(value).replace(/%/g, ''))
  if (n == null || !Number.isFinite(n)) return ''
  return roundPercent(n).toFixed(PERCENT_DECIMALS)
}

/** Converte texto pt-BR ou canônico em número (sem símbolo %). */
export function parsePercentFromDisplay(text: string): number | null {
  const trimmed = text.trim().replace(/%/g, '')
  if (trimmed === '') return null
  let s = trimmed.replace(/\s/g, '')
  if (s.includes(',') && s.includes('.')) {
    s = s.replace(/\./g, '').replace(',', '.')
  } else if (s.includes(',')) {
    s = s.replace(',', '.')
  }
  const n = Number(s)
  return Number.isFinite(n) ? n : null
}

/** Exibição pt-BR (até 4 decimais, sem sufixo % — rótulo já indica). */
export function formatPercentForDisplay(formValue: string | undefined): string {
  if (formValue == null || formValue.trim() === '') return ''
  const n = Number(formValue)
  if (!Number.isFinite(n)) {
    const parsed = parsePercentFromDisplay(formValue)
    if (parsed == null) return formValue
    return parsed.toLocaleString('pt-BR', {
      minimumFractionDigits: 2,
      maximumFractionDigits: PERCENT_DECIMALS,
    })
  }
  return n.toLocaleString('pt-BR', {
    minimumFractionDigits: 2,
    maximumFractionDigits: PERCENT_DECIMALS,
  })
}

/** Exibição read-only com sufixo % (cards/show). */
export function formatPercentValueForDisplay(value: unknown): string {
  if (value == null || value === '') return '—'
  const canonical = normalizePercentToForm(value)
  if (canonical === '') return '—'
  return `${formatPercentForDisplay(canonical)}%`
}

/** Valor canônico do form → número para JSON API. */
export function percentFormToNumber(raw: string): number | null {
  if (raw.trim() === '') return null
  const n = Number(raw)
  if (Number.isFinite(n)) return roundPercent(n)
  const parsed = parsePercentFromDisplay(raw)
  return parsed == null ? null : roundPercent(parsed)
}
