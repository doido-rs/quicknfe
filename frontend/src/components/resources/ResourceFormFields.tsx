import type { Dispatch, FormEvent, SetStateAction } from 'react'
import { Link } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import type { FieldDef } from '@/resources/definitions'

export function ResourceFormFields({
  fields,
  form,
  setForm,
  onSubmit,
  submitLabel,
  pending,
  cancelTo,
  onCancel,
}: {
  fields: FieldDef[]
  form: Record<string, string>
  setForm: Dispatch<SetStateAction<Record<string, string>>>
  onSubmit: (e: FormEvent) => void
  submitLabel: string
  pending?: boolean
  cancelTo?: string
  onCancel?: () => void
}) {
  return (
    <form onSubmit={onSubmit} className="grid gap-4 md:grid-cols-2">
      {fields.map((field) => (
        <div key={field.key} className="grid gap-2">
          <Label htmlFor={`field-${field.key}`}>{field.label}</Label>
          <Input
            id={`field-${field.key}`}
            required={field.required}
            type={
              field.type === 'number'
                ? 'number'
                : field.type === 'date'
                  ? 'datetime-local'
                  : 'text'
            }
            value={form[field.key] ?? ''}
            onChange={(ev) => setForm((prev) => ({ ...prev, [field.key]: ev.target.value }))}
          />
        </div>
      ))}
      <div className="flex flex-wrap gap-2 md:col-span-2">
        <Button type="submit" disabled={pending}>
          {submitLabel}
        </Button>
        {cancelTo ? (
          <Button type="button" variant="outline" asChild>
            <Link to={cancelTo}>Cancelar</Link>
          </Button>
        ) : onCancel ? (
          <Button type="button" variant="outline" onClick={onCancel}>
            Cancelar
          </Button>
        ) : null}
      </div>
    </form>
  )
}
