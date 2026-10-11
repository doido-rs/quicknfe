import { useForm, type UseFormReturn } from 'react-hook-form'
import type { FieldDef } from '../resources/definitions'

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
    fields.map((f) => [f.key, row[f.key] != null ? String(row[f.key]) : '']),
  )
}

export function formatFieldValue(value: unknown, field?: FieldDef): string {
  if (value == null || value === '') return '—'
  if (field?.enumOptions?.length) {
    const match = field.enumOptions.find((o) => o.value === value)
    if (match) return match.label
  }
  return String(value)
}

export function buildBody(fields: FieldDef[], form: Record<string, string>) {
  return Object.fromEntries(
    fields.filter((f) => !f.readOnly).map((f) => {
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
