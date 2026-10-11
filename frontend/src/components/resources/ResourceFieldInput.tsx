import type { UseFormReturn } from 'react-hook-form'
import { Controller } from 'react-hook-form'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ForeignKeyAutocomplete } from '@/components/resources/ForeignKeyAutocomplete'
import { relationResourceKey } from '@/lib/relations'
import type { FieldDef } from '@/resources/definitions'

export function ResourceFieldInput({
  field,
  form,
  idPrefix = 'field',
  attachedFile,
  onFileChange,
}: {
  field: FieldDef
  form: UseFormReturn<Record<string, string>>
  idPrefix?: string
  attachedFile?: File
  onFileChange?: (key: string, file: File | null) => void
}) {
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
        type={field.type === 'number' ? 'number' : field.type === 'date' ? 'date' : 'text'}
        {...form.register(field.key, { required: field.required && !field.readOnly })}
      />
    </div>
  )
}
