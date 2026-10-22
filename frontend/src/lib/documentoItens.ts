import { normalizeMoneyToForm, roundMoney } from '@/lib/moneyFormat'

/** Soma `valor_total_bruto` dos itens preparados para a API. */
export function sumDocumentoItensValor(itens: Record<string, unknown>[]): number {
  return roundMoney(
    itens.reduce((acc, item) => {
      const n = Number(item.valor_total_bruto)
      return acc + (Number.isFinite(n) ? n : 0)
    }, 0),
  )
}

/** Atualiza valor da operação no form quando há itens com total > 0. */
export function operacaoFormValueFromItens(
  itens: Record<string, unknown>[],
  savedOperacaoFromRow: unknown,
): { value: string; shouldDirty: boolean } | null {
  if (itens.length === 0) return null
  const sum = sumDocumentoItensValor(itens)
  if (sum <= 0) return null
  const value = sum.toFixed(2)
  const saved = normalizeMoneyToForm(savedOperacaoFromRow)
  return { value, shouldDirty: value !== saved }
}

function parseAmount(raw: string | undefined): number {
  if (raw == null || raw.trim() === '') return 0
  const n = Number(raw)
  return Number.isFinite(n) ? n : 0
}

/** Alinha valor da operação e líquido com a soma dos itens rascunho. */
export function applyItensToDocumentoBody(
  body: Record<string, unknown>,
  formValues: Record<string, string>,
  itens: Record<string, unknown>[],
): void {
  if (itens.length === 0) return
  const sum = sumDocumentoItensValor(itens)
  if (sum <= 0) return
  body.valor_total_servicos_produtos = sum
  const descIncond = parseAmount(formValues.desconto_incondicionado)
  const descCond = parseAmount(formValues.desconto_condicionado)
  body.valor_liquido = Math.max(0, sum - descIncond - descCond)
}
