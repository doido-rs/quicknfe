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
}: {
  field: FieldDef
  form: UseFormReturn<Record<string, string>>
  idPrefix?: string
}) {
  const relationKey = relationResourceKey(field)

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

  const inputId = `${idPrefix}-${field.key}`

  return (
    <div className="grid gap-2">
      <Label htmlFor={inputId}>{field.label}</Label>
      <Input
        id={inputId}
        readOnly={field.readOnly}
        className={field.readOnly ? 'bg-muted' : undefined}
        type={
          field.type === 'number'
            ? 'number'
            : field.type === 'date'
              ? 'datetime-local'
              : 'text'
        }
        {...form.register(field.key, { required: field.required && !field.readOnly })}
      />
    </div>
  )
}
