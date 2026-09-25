import type { FieldDef } from '../resources/definitions'

export function emptyForm(fields: FieldDef[]): Record<string, string> {
  return Object.fromEntries(fields.map((f) => [f.key, '']))
}

export function rowLabel(row: Record<string, unknown>, fields: FieldDef[]): string {
  const first = fields.find((f) => row[f.key] != null)
  return first ? String(row[first.key]) : String(row.id ?? '')
}

export function buildBody(fields: FieldDef[], form: Record<string, string>) {
  return Object.fromEntries(
    fields.map((f) => {
      const raw = form[f.key]
      if (f.type === 'number' && raw !== '') {
        return [f.key, Number(raw)]
      }
      if (raw === '') {
        return [f.key, null]
      }
      return [f.key, raw]
    }),
  )
}
