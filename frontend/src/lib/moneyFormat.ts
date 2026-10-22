/**
 * Campos monetários (R$) do app — tipo `money` em `definitions.ts`.
 * Estado do formulário: string canônica com ponto e 2 decimais (`"1234.56"`).
 */
export const MONEY_FIELD_KEYS = [
  'valor_total_servicos_produtos',
  'valor_recebido_intermediario',
  'desconto_incondicionado',
  'desconto_condicionado',
  'valor_liquido',
  'issqn_base_calculo',
  'issqn_valor',
  'valor_irrf',
  'valor_contrib_sociais_retidas',
  'valor_contrib_previdenciaria_retida',
  'valor_unitario',
  'valor_desconto',
  'valor_total_bruto',
  'base_calculo',
  'valor_imposto',
] as const

export type MoneyFieldKey = (typeof MONEY_FIELD_KEYS)[number]

export function isMoneyFieldKey(key: string): key is MoneyFieldKey {
  return (MONEY_FIELD_KEYS as readonly string[]).includes(key)
}

/** API / número → valor canônico do form (`1234.56`). */
export function normalizeMoneyToForm(value: unknown): string {
  if (value == null || value === '') return ''
  if (typeof value === 'string' && value.trim() === '') return ''
  const n = typeof value === 'number' ? value : parseMoneyFromDisplay(String(value))
  if (n == null || !Number.isFinite(n)) return ''
  return roundMoney(n).toFixed(2)
}

export function roundMoney(n: number): number {
  return Math.round(n * 100) / 100
}

/** Converte texto pt-BR ou canônico em número. */
export function parseMoneyFromDisplay(text: string): number | null {
  const trimmed = text.trim()
  if (trimmed === '') return null
  let s = trimmed.replace(/\s/g, '').replace(/^R\$/i, '')
  if (s.includes(',') && s.includes('.')) {
    s = s.replace(/\./g, '').replace(',', '.')
  } else if (s.includes(',')) {
    s = s.replace(',', '.')
  }
  const n = Number(s)
  return Number.isFinite(n) ? n : null
}

/** Exibição pt-BR com 2 decimais. */
export function formatMoneyForDisplay(formValue: string | undefined): string {
  if (formValue == null || formValue.trim() === '') return ''
  const n = Number(formValue)
  if (!Number.isFinite(n)) {
    const parsed = parseMoneyFromDisplay(formValue)
    if (parsed == null) return formValue
    return parsed.toLocaleString('pt-BR', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })
  }
  return n.toLocaleString('pt-BR', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })
}

/** Valor canônico do form → número para JSON API. */
export function moneyFormToNumber(raw: string): number | null {
  if (raw.trim() === '') return null
  const n = Number(raw)
  if (Number.isFinite(n)) return roundMoney(n)
  const parsed = parseMoneyFromDisplay(raw)
  return parsed == null ? null : roundMoney(parsed)
}
