import type { UseFormReturn } from 'react-hook-form'
import { Controller } from 'react-hook-form'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { cn } from '@/lib/utils'
import { ForeignKeyAutocomplete } from '@/components/resources/ForeignKeyAutocomplete'
import { relationResourceKey } from '@/lib/relations'
import {
  formDateToDisplay,
  parseDateFromDisplay,
} from '@/lib/dateFormat'
import { readOnlyFieldDisplay } from '@/lib/documentoShowLabels'
import { useDateFormat } from '@/theme/DateFormatContext'
import type { FieldDef } from '@/resources/definitions'

export function ResourceFieldInput({
  field,
  form,
  idPrefix = 'field',
  attachedFile,
  onFileChange,
  readOnly,
  relationDisplayLabels,
  displayRow,
}: {
  field: FieldDef
  form: UseFormReturn<Record<string, string>>
  idPrefix?: string
  attachedFile?: File
  onFileChange?: (key: string, file: File | null) => void
  readOnly?: boolean
  relationDisplayLabels?: Record<string, string>
  displayRow?: Record<string, unknown>
}) {
  const { preference: dateFormat } = useDateFormat()

  if (readOnly) {
    const raw = form.getValues(field.key)
    const text = readOnlyFieldDisplay(
      field,
      raw ?? '',
      displayRow ?? {},
      relationDisplayLabels,
      dateFormat,
    )
    return (
      <div className="grid gap-1.5">
        <Label className="text-muted-foreground">{field.label}</Label>
        <p className="text-sm leading-snug whitespace-pre-wrap">{text || '—'}</p>
      </div>
    )
  }

  const relationKey = relationResourceKey(field)
  const scopeField = field.relationScopeFromField
  const scopeRaw = scopeField ? form.watch(scopeField) : ''
  const scopeEmpresaId =
    scopeRaw.trim() === '' || scopeRaw == null ? undefined : Number(scopeRaw)

  if (relationKey) {
    return (
      <Controller
        name={field.key}
        control={form.control}
        rules={{ required: field.required }}
        render={({ field: rhf }) => (
          <ForeignKeyAutocomplete
            field={field}
            resourceKey={relationKey}
            value={rhf.value ?? ''}
            onChange={rhf.onChange}
            listOptions={
              field.relationScopeFromField && scopeEmpresaId != null && Number.isFinite(scopeEmpresaId)
                ? { empresaId: scopeEmpresaId }
                : undefined
            }
            onSelectRow={(row) => {
              if (!field.relationCopyTo) return
              for (const [formKey, rowKey] of Object.entries(field.relationCopyTo)) {
                const v = row[rowKey]
                form.setValue(formKey, v != null ? String(v) : '')
              }
            }}
            required={field.required}
          />
        )}
      />
    )
  }

  if (field.type === 'boolean') {
    const checkboxId = `${idPrefix}-${field.key}`
    return (
      <div className="flex items-center gap-2 md:col-span-2">
        <input
          id={checkboxId}
          type="checkbox"
          className="size-4 rounded border border-input"
          checked={form.watch(field.key) === 'true'}
          onChange={(e) => form.setValue(field.key, e.target.checked ? 'true' : 'false')}
        />
        <Label htmlFor={checkboxId} className="font-normal leading-snug">
          {field.label}
        </Label>
      </div>
    )
  }

  if (field.type === 'enum' && field.enumOptions?.length) {
    const selectId = `${idPrefix}-${field.key}`
    return (
      <div className="grid gap-2">
        <Label htmlFor={selectId}>{field.label}</Label>
        <select
          id={selectId}
          className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
          {...form.register(field.key, { required: field.required })}
        >
          <option value="" disabled={field.required}>
            {field.required ? 'Selecione…' : '—'}
          </option>
          {field.enumOptions.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>
    )
  }

  const inputId = `${idPrefix}-${field.key}`

  if (field.type === 'date') {
    const ymd = form.watch(field.key) ?? ''
    const displayValue = formDateToDisplay(ymd, dateFormat)
    return (
      <div className="grid gap-2">
        <Label htmlFor={inputId}>{field.label}</Label>
        <Input
          id={inputId}
          readOnly={field.readOnly}
          className={field.readOnly ? 'bg-muted' : undefined}
          type="text"
          inputMode="numeric"
          placeholder={
            dateFormat === 'DMY' ? 'DD/MM/AAAA' : dateFormat === 'MDY' ? 'MM/DD/AAAA' : 'AAAA-MM-DD'
          }
          value={displayValue}
          onChange={(e) => {
            const parsed = parseDateFromDisplay(e.target.value, dateFormat)
            form.setValue(field.key, parsed ?? e.target.value, { shouldDirty: true })
          }}
          onBlur={(e) => {
            const parsed = parseDateFromDisplay(e.target.value, dateFormat)
            if (parsed) form.setValue(field.key, parsed, { shouldDirty: true })
          }}
        />
      </div>
    )
  }

  if (field.type === 'textarea') {
    return (
      <div className="grid gap-2 md:col-span-2">
        <Label htmlFor={inputId}>{field.label}</Label>
        <textarea
          id={inputId}
          rows={4}
          readOnly={field.readOnly}
          className={cn(
            'w-full min-w-0 rounded-lg border border-input bg-transparent px-2.5 py-2 text-base transition-colors outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:bg-input/50 disabled:opacity-50 md:text-sm dark:bg-input/30',
            field.readOnly && 'bg-muted',
          )}
          {...form.register(field.key, { required: field.required && !field.readOnly })}
        />
      </div>
    )
  }

  if (field.type === 'file') {
    return (
      <div className="grid gap-2 md:col-span-2">
        <Label htmlFor={inputId}>{field.label}</Label>
        <Input
          id={inputId}
          type="file"
          accept={field.accept}
          required={field.required && !attachedFile}
          onChange={(e) => {
            const file = e.target.files?.[0] ?? null
            onFileChange?.(field.key, file)
          }}
        />
        {attachedFile && (
          <p className="text-xs text-muted-foreground">
            Selecionado: {attachedFile.name} ({Math.ceil(attachedFile.size / 1024)} KB)
          </p>
        )}
      </div>
    )
  }

  return (
    <div className="grid gap-2">
      <Label htmlFor={inputId}>{field.label}</Label>
      <Input
        id={inputId}
        readOnly={field.readOnly}
        className={field.readOnly ? 'bg-muted' : undefined}
        type={field.type === 'number' ? 'number' : 'text'}
        {...form.register(field.key, { required: field.required && !field.readOnly })}
      />
    </div>
  )
}
