import { formatFieldValue } from '@/lib/resourceForm'
import type { FieldDef } from '@/resources/definitions'

function formatMunicipio(m: Record<string, unknown>): string {
  const nome = m.nome != null ? String(m.nome) : ''
  const uf = m.uf != null ? String(m.uf) : ''
  const codigo = m.codigo_ibge != null ? String(m.codigo_ibge) : ''
  if (nome && uf) return `${nome} — ${uf}${codigo ? ` (${codigo})` : ''}`
  return codigo || nome || '—'
}

/** Rótulos para campos relation no formulário somente leitura da nota. */
export function buildDocumentoRelationLabels(
  row: Record<string, unknown>,
): Record<string, string> {
  const labels: Record<string, string> = {}

  if (row.prestador_razao_social != null && String(row.prestador_razao_social).trim()) {
    labels.empresa_id = String(row.prestador_razao_social)
  }
  if (row.tomador_razao_social != null && String(row.tomador_razao_social).trim()) {
    labels.cliente_id = String(row.tomador_razao_social)
  }
  if (row.intermediario_razao_social != null && String(row.intermediario_razao_social).trim()) {
    labels.intermediario_id = String(row.intermediario_razao_social)
  }

  const emissao = row.municipio_emissao
  if (emissao != null && typeof emissao === 'object') {
    labels.municipio_emissao_id = formatMunicipio(emissao as Record<string, unknown>)
  }
  const prestacao = row.municipio_prestacao
  if (prestacao != null && typeof prestacao === 'object') {
    labels.municipio_prestacao_id = formatMunicipio(prestacao as Record<string, unknown>)
  }

  return labels
}

export function readOnlyFieldDisplay(
  field: FieldDef,
  formValue: string,
  row: Record<string, unknown>,
  relationLabels?: Record<string, string>,
): string {
  const fromRelation = relationLabels?.[field.key]
  if (fromRelation) return fromRelation
  return formatFieldValue(formValue, field, row)
}
