export function parseDocumentoAmount(raw: string | undefined): number {
  if (raw == null || raw.trim() === '') return 0
  const n = Number(raw)
  return Number.isFinite(n) ? n : 0
}

/** Mesma regra de `buildDocumentoBody` para valor líquido. */
export function computeDocumentoValorLiquido(values: Record<string, string>): number {
  const operacao = parseDocumentoAmount(values.valor_total_servicos_produtos)
  const descIncond = parseDocumentoAmount(values.desconto_incondicionado)
  const descCond = parseDocumentoAmount(values.desconto_condicionado)
  return Math.max(0, operacao - descIncond - descCond)
}
