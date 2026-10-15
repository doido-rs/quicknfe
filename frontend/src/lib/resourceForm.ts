import { useForm, type UseFormReturn } from 'react-hook-form'
import {
  formatDateForDisplay,
  normalizeApiDateToForm,
  type DateFormatPreference,
} from '@/lib/dateFormat'
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
  return Object.fromEntries(
    fields.map((f) => [f.key, f.defaultValue ?? '']),
  )
}

function formStringValue(raw: unknown, field: FieldDef): string {
  if (field.type === 'boolean') {
    if (raw === true || raw === 'true' || raw === 1 || raw === '1') return 'true'
    return 'false'
  }
  if (raw == null) return ''
  if (field.type === 'date') {
    return normalizeApiDateToForm(raw)
  }
  return String(raw)
}

export function rowToForm(row: Record<string, unknown>, fields: FieldDef[]): Record<string, string> {
  return Object.fromEntries(
    fields.map((f) => {
      const raw = fieldValue(row, f.key)
      if (raw == null && f.defaultValue != null) {
        return [f.key, f.defaultValue]
      }
      return [f.key, formStringValue(raw, f)]
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
  dateFormat?: DateFormatPreference,
): string {
  if (dateFormat && (field?.type === 'date' || (typeof value === 'string' && value.includes('T')))) {
    const formatted = formatDateForDisplay(value, dateFormat)
    if (formatted !== '—') return formatted
  }
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

function isFileField(f: FieldDef) {
  return f.type === 'file'
}

export function fileToBase64Payload(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const result = reader.result
      if (typeof result !== 'string') {
        reject(new Error('leitura do arquivo falhou'))
        return
      }
      const comma = result.indexOf(',')
      resolve(comma >= 0 ? result.slice(comma + 1) : result)
    }
    reader.onerror = () => reject(reader.error ?? new Error('leitura do arquivo falhou'))
    reader.readAsDataURL(file)
  })
}

/** Doido `body_json` / certificado storage — keep JSON under ~2 MiB total. */
export const MAX_UPLOAD_FILE_BYTES = 1_400_000

export async function buildCreateBodyWithFiles(
  fields: FieldDef[],
  form: Record<string, string>,
  files: Record<string, File | undefined>,
): Promise<Record<string, unknown>> {
  for (const field of fields) {
    if (field.type !== 'file') continue
    const file = files[field.key]
    if (file && file.size > MAX_UPLOAD_FILE_BYTES) {
      throw new Error(
        `Arquivo muito grande (${Math.ceil(file.size / 1024)} KB). Limite ~${Math.floor(MAX_UPLOAD_FILE_BYTES / 1024)} KB para upload via JSON.`,
      )
    }
  }

  const body = buildBody(
    fields.filter((f) => !isFileField(f)),
    form,
  ) as Record<string, unknown>

  for (const field of fields) {
    if (!isFileField(field) || !field.fileUploadAs) continue
    const file = files[field.key]
    if (!file) continue
    body[field.fileUploadAs.base64Key] = await fileToBase64Payload(file)
    body[field.fileUploadAs.nameKey] = file.name
  }

  return body
}

/** Corpo JSON para POST de recurso aninhado (ex.: item da nota). */
export function buildNestedCreateBody(
  nested: { fields: FieldDef[]; prepareCreateBody?: (body: Record<string, unknown>) => Record<string, unknown> },
  values: Record<string, string>,
): Record<string, unknown> {
  let body = buildBody(nested.fields, values) as Record<string, unknown>
  if (nested.prepareCreateBody) {
    body = nested.prepareCreateBody(body)
  }
  return body
}

export function buildBody(fields: FieldDef[], form: Record<string, string>) {
  return Object.fromEntries(
    fields
      .filter((f) => !f.readOnly && !f.key.includes('.') && !isFileField(f))
      .map((f) => {
      const raw = form[f.key]
      if (f.type === 'boolean') {
        return [f.key, raw === 'true' || raw === '1']
      }
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
  const preferredKeys = ['descricao_servico', 'descricao', 'razao_social', 'nome']
  for (const key of preferredKeys) {
    if (fields.some((f) => f.key === key) && row[key] != null && String(row[key]).trim() !== '') {
      return String(row[key])
    }
  }
  const first = fields.find((f) => row[f.key] != null && String(row[f.key]).trim() !== '')
  return first ? String(row[first.key]) : String(row.id ?? '')
}
