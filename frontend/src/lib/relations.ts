import { rowLabel } from '@/lib/resourceForm'
import { resources, type FieldDef } from '@/resources/definitions'

const FK_RESOURCE: Record<string, string> = {
  cliente_id: 'clientes',
  intermediario_id: 'intermediarios',
  lote_id: 'lotes',
  documento_fiscal_id: 'documentos',
  municipio_id: 'municipios',
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
  const def = resources.find((r) => r.key === resourceKey)
  if (!def) return `#${String(row.id ?? '')}`
  const summary = rowLabel(row, def.fields)
  return summary ? `${summary} (#${row.id})` : `#${row.id}`
}
