import { useForm, type UseFormReturn } from 'react-hook-form'
import type { FieldDef } from '../resources/definitions'

export function fieldValue(row: Record<string, unknown>, key: string): unknown {
  if (!key.includes('.')) return row[key]
  let current: unknown = row
  for (const part of key.split('.')) {
    if (current == null || typeof current !== 'object') return undefined
    current = (current as Record<string, unknown>)[part]
  }
  return current
}

export function fieldsByKeys(all: FieldDef[], keys: string[]): FieldDef[] {
  return keys
    .map((key) => all.find((f) => f.key === key))
    .filter((f): f is FieldDef => f != null)
}

export function emptyForm(fields: FieldDef[]): Record<string, string> {
  return Object.fromEntries(fields.map((f) => [f.key, '']))
}

export function rowToForm(row: Record<string, unknown>, fields: FieldDef[]): Record<string, string> {
  return Object.fromEntries(
    fields.map((f) => {
      const raw = fieldValue(row, f.key)
      return [f.key, raw != null ? String(raw) : '']
    }),
  )
}

export function formatMunicipioRelationLabel(row: Record<string, unknown>): string | null {
  const m = row.municipio
  if (m == null || typeof m !== 'object') return null
  const { nome, uf, codigo_ibge: codigo } = m as {
    nome?: string
    uf?: string
    codigo_ibge?: string
  }
  if (!nome) return null
  const tail = [uf, codigo ? `IBGE ${codigo}` : ''].filter(Boolean).join(' · ')
  return tail ? `${nome} (${tail})` : nome
}

export function formatFieldValue(
  value: unknown,
  field?: FieldDef,
  row?: Record<string, unknown>,
): string {
  if (field?.relationResource === 'municipios' && row) {
    const fromNested = formatMunicipioRelationLabel(row)
    if (fromNested) return fromNested
  }
  if (value == null || value === '') return '—'
  if (field?.enumOptions?.length) {
    const match = field.enumOptions.find((o) => o.value === value)
    if (match) return match.label
  }
  return String(value)
}

export function buildBody(fields: FieldDef[], form: Record<string, string>) {
  return Object.fromEntries(
    fields.filter((f) => !f.readOnly && !f.key.includes('.')).map((f) => {
      const raw = form[f.key]
      if (raw !== '' && (f.type === 'number' || f.key.endsWith('_id'))) {
        return [f.key, Number(raw)]
      }
      if (raw === '') {
        return [f.key, null]
      }
      return [f.key, raw]
    }),
  )
}

export function useResourceFieldForm(
  fields: FieldDef[],
  defaultValues?: Record<string, string>,
): UseFormReturn<Record<string, string>> {
  return useForm<Record<string, string>>({
    defaultValues: defaultValues ?? emptyForm(fields),
  })
}

export function rowLabel(row: Record<string, unknown>, fields: FieldDef[]): string {
  const first = fields.find((f) => row[f.key] != null)
  return first ? String(row[first.key]) : String(row.id ?? '')
}
