import { useState } from 'react'
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
import {
  formatMoneyForDisplay,
  parseMoneyFromDisplay,
  roundMoney,
} from '@/lib/moneyFormat'
import {
  formatPercentForDisplay,
  parsePercentFromDisplay,
  PERCENT_DECIMALS,
  roundPercent,
} from '@/lib/percentFormat'
import { readOnlyFieldDisplay } from '@/lib/documentoShowLabels'
import { useDateFormat } from '@/theme/DateFormatContext'
import type { FieldDef } from '@/resources/definitions'

function PercentFieldInput({
  field,
  form,
  inputId,
  dense,
  readOnly,
}: {
  field: FieldDef
  form: UseFormReturn<Record<string, string>>
  inputId: string
  dense?: boolean
  readOnly?: boolean
}) {
  const [editing, setEditing] = useState(false)
  const isReadOnly = readOnly || field.readOnly

  return (
    <Controller
      name={field.key}
      control={form.control}
      rules={{
        required: field.required && !isReadOnly ? `${field.label} é obrigatório.` : false,
      }}
      render={({ field: controlField, fieldState }) => {
        const canonical = controlField.value ?? ''
        const displayValue = isReadOnly
          ? formatPercentForDisplay(canonical)
          : editing
            ? canonical.replace('.', ',')
            : formatPercentForDisplay(canonical)

        return (
          <div className={cn('grid gap-2', dense && 'gap-1.5')}>
            <Label htmlFor={inputId} className={dense ? 'text-xs' : undefined}>
              {field.label}
            </Label>
            <Input
              id={inputId}
              readOnly={isReadOnly}
              className={cn(isReadOnly && 'bg-muted', dense && 'h-8 text-sm')}
              type="text"
              inputMode="decimal"
              placeholder="0,00"
              aria-invalid={!!fieldState.error}
              value={displayValue}
              onChange={(e) => {
                if (isReadOnly) return
                controlField.onChange(e.target.value.replace(',', '.').replace(/%/g, ''))
              }}
              onFocus={() => {
                if (!isReadOnly) setEditing(true)
              }}
              onBlur={(e) => {
                if (isReadOnly) return
                setEditing(false)
                const parsed = parsePercentFromDisplay(e.target.value)
                if (parsed != null) {
                  controlField.onChange(roundPercent(parsed).toFixed(PERCENT_DECIMALS))
                } else if (e.target.value.trim() === '') {
                  controlField.onChange('')
                }
                controlField.onBlur()
              }}
            />
            {fieldState.error?.message && (
              <p className="text-sm text-destructive">{fieldState.error.message}</p>
            )}
          </div>
        )
      }}
    />
  )
}

function MoneyFieldInput({
  field,
  form,
  inputId,
  dense,
  readOnly,
}: {
  field: FieldDef
  form: UseFormReturn<Record<string, string>>
  inputId: string
  dense?: boolean
  readOnly?: boolean
}) {
  const [editing, setEditing] = useState(false)
  const isReadOnly = readOnly || field.readOnly

  return (
    <Controller
      name={field.key}
      control={form.control}
      rules={{
        required: field.required && !isReadOnly ? `${field.label} é obrigatório.` : false,
      }}
      render={({ field: controlField, fieldState }) => {
        const canonical = controlField.value ?? ''
        const displayValue = isReadOnly
          ? formatMoneyForDisplay(canonical)
          : editing
            ? canonical.replace('.', ',')
            : formatMoneyForDisplay(canonical)

        return (
          <div className={cn('grid gap-2', dense && 'gap-1.5')}>
            <Label htmlFor={inputId} className={dense ? 'text-xs' : undefined}>
              {field.label}
            </Label>
            <Input
              id={inputId}
              readOnly={isReadOnly}
              className={cn(isReadOnly && 'bg-muted', dense && 'h-8 text-sm')}
              type="text"
              inputMode="decimal"
              placeholder="0,00"
              aria-invalid={!!fieldState.error}
              value={displayValue}
              onChange={(e) => {
                if (isReadOnly) return
                controlField.onChange(e.target.value.replace(',', '.'))
              }}
              onFocus={() => {
                if (!isReadOnly) setEditing(true)
              }}
              onBlur={(e) => {
                if (isReadOnly) return
                setEditing(false)
                const parsed = parseMoneyFromDisplay(e.target.value)
                if (parsed != null) {
                  controlField.onChange(roundMoney(parsed).toFixed(2))
                } else if (e.target.value.trim() === '') {
                  controlField.onChange('')
                }
                controlField.onBlur()
              }}
            />
            {fieldState.error?.message && (
              <p className="text-sm text-destructive">{fieldState.error.message}</p>
            )}
          </div>
        )
      }}
    />
  )
}

export function ResourceFieldInput({
  field,
  form,
  idPrefix = 'field',
  attachedFile,
  onFileChange,
  readOnly,
  relationDisplayLabels,
  displayRow,
  relationScopeValues,
  dense,
}: {
  field: FieldDef
  form: UseFormReturn<Record<string, string>>
  idPrefix?: string
  attachedFile?: File
  onFileChange?: (key: string, file: File | null) => void
  readOnly?: boolean
  relationDisplayLabels?: Record<string, string>
  displayRow?: Record<string, unknown>
  relationScopeValues?: Record<string, string>
  /** Campos menores (ex.: itens da nota). */
  dense?: boolean
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
  const scopeRaw = scopeField
    ? (relationScopeValues?.[scopeField] ?? form.watch(scopeField))
    : ''
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
    const isoDatePattern = /^\d{4}-\d{2}-\d{2}$/
    const validateIso = (value: string | undefined) => {
      if (!field.required || field.readOnly) return true
      const trimmed = (value ?? '').trim()
      if (!trimmed) return `${field.label} é obrigatório.`
      if (!isoDatePattern.test(trimmed)) return 'Informe uma data válida.'
      const [y, m, d] = trimmed.split('-').map(Number)
      const dt = new Date(y, m - 1, d)
      if (dt.getFullYear() !== y || dt.getMonth() !== m - 1 || dt.getDate() !== d) {
        return 'Informe uma data válida.'
      }
      return true
    }
    return (
      <Controller
        name={field.key}
        control={form.control}
        rules={{
          required: field.required && !field.readOnly ? `${field.label} é obrigatório.` : false,
          validate: validateIso,
        }}
        render={({ field: controlField, fieldState }) => (
          <div className="grid gap-2">
            <Label htmlFor={inputId}>{field.label}</Label>
            <Input
              id={inputId}
              readOnly={field.readOnly}
              className={field.readOnly ? 'bg-muted' : undefined}
              type="text"
              inputMode="numeric"
              placeholder={
                dateFormat === 'DMY'
                  ? 'DD/MM/AAAA'
                  : dateFormat === 'MDY'
                    ? 'MM/DD/AAAA'
                    : 'AAAA-MM-DD'
              }
              aria-invalid={!!fieldState.error}
              value={formDateToDisplay(controlField.value ?? '', dateFormat)}
              onChange={(e) => {
                const parsed = parseDateFromDisplay(e.target.value, dateFormat)
                controlField.onChange(parsed ?? e.target.value)
              }}
              onBlur={(e) => {
                const parsed = parseDateFromDisplay(e.target.value, dateFormat)
                if (parsed) controlField.onChange(parsed)
                controlField.onBlur()
              }}
            />
            {fieldState.error?.message && (
              <p className="text-sm text-destructive">{fieldState.error.message}</p>
            )}
          </div>
        )}
      />
    )
  }

  if (field.type === 'textarea') {
    return (
      <div className={cn('grid gap-2', dense ? 'gap-1.5' : 'md:col-span-2')}>
        <Label htmlFor={inputId} className={dense ? 'text-xs' : undefined}>
          {field.label}
        </Label>
        <textarea
          id={inputId}
          rows={dense ? 2 : 4}
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

  if (field.type === 'percent') {
    return (
      <PercentFieldInput
        field={field}
        form={form}
        inputId={inputId}
        dense={dense}
        readOnly={readOnly}
      />
    )
  }

  if (field.type === 'money') {
    return (
      <MoneyFieldInput
        field={field}
        form={form}
        inputId={inputId}
        dense={dense}
        readOnly={readOnly}
      />
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
    <div className={cn('grid gap-2', dense && 'gap-1.5')}>
      <Label htmlFor={inputId} className={dense ? 'text-xs' : undefined}>
        {field.label}
      </Label>
      <Input
        id={inputId}
        readOnly={field.readOnly}
        className={cn(field.readOnly && 'bg-muted', dense && 'h-8 text-sm')}
        type={
          field.inputType === 'password'
            ? 'password'
            : field.type === 'number'
              ? 'number'
              : 'text'
        }
        {...form.register(field.key, { required: field.required && !field.readOnly })}
      />
    </div>
  )
}
