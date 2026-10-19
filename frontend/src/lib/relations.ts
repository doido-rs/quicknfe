import { rowLabel } from '@/lib/resourceForm'
import { resources, type FieldDef } from '@/resources/definitions'

const FK_RESOURCE: Record<string, string> = {
  cliente_id: 'clientes',
  intermediario_id: 'intermediarios',
  lote_id: 'lotes',
  documento_fiscal_id: 'documentos',
  municipio_id: 'municipios',
  cna_id: 'cnas',
  conta_fiscal_id: 'contas-fiscais',
  empresa_id: 'prestadores',
}

export function relationResourceKey(field: FieldDef): string | null {
  if (field.type === 'relation' && field.relationResource) {
    return field.relationResource
  }
  return FK_RESOURCE[field.key] ?? null
}

export function relationApiPath(resourceKey: string): string | undefined {
  return resources.find((r) => r.key === resourceKey)?.path
}

export function relationOptionLabel(resourceKey: string, row: Record<string, unknown>): string {
  if (resourceKey === 'contas-fiscais') {
    const codigo = row.codigo != null ? String(row.codigo) : ''
    const nac = row.codigo_tributacao_nacional != null ? String(row.codigo_tributacao_nacional) : ''
    const mun =
      row.codigo_tributacao_municipal != null ? String(row.codigo_tributacao_municipal) : ''
    const parts = [codigo, nac && mun ? `${nac}/${mun}` : nac].filter(Boolean)
    return parts.length > 0 ? parts.join(' · ') : `#${String(row.id ?? '')}`
  }
  if (resourceKey === 'cnas') {
    const codigo = row.codigo != null ? String(row.codigo) : ''
    const descricao = row.descricao != null ? String(row.descricao) : ''
    if (codigo && descricao) return `${codigo} — ${descricao}`
    return codigo || descricao || `#${String(row.id ?? '')}`
  }
  const def = resources.find((r) => r.key === resourceKey)
  if (!def) return `#${String(row.id ?? '')}`
  const summary = rowLabel(row, def.fields)
  return summary ? `${summary} (#${row.id})` : `#${row.id}`
}
