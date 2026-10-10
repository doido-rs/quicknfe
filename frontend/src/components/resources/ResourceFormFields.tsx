import { Link } from '@tanstack/react-router'
import type { UseFormReturn } from 'react-hook-form'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import type { ResourceRouteLink } from '@/lib/resourcePaths'
import type { FieldDef } from '@/resources/definitions'

export function ResourceFormFields({
  fields,
  form,
  onSubmit,
  submitLabel,
  cancelLink,
  onCancel,
}: {
  fields: FieldDef[]
  form: UseFormReturn<Record<string, string>>
  onSubmit: (values: Record<string, string>) => void | Promise<void>
  submitLabel: string
  cancelLink?: ResourceRouteLink
  onCancel?: () => void
}) {
  const {
    register,
    handleSubmit,
    formState: { isSubmitting },
  } = form

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="grid gap-4 md:grid-cols-2">
      {fields.map((field) => (
        <div key={field.key} className="grid gap-2">
          <Label htmlFor={`field-${field.key}`}>{field.label}</Label>
          <Input
            id={`field-${field.key}`}
            type={
              field.type === 'number'
                ? 'number'
                : field.type === 'date'
                  ? 'datetime-local'
                  : 'text'
            }
            {...register(field.key, { required: field.required })}
          />
        </div>
      ))}
      <div className="flex flex-wrap gap-2 md:col-span-2">
        <Button type="submit" disabled={isSubmitting}>
          {submitLabel}
        </Button>
        {cancelLink ? (
          <Button type="button" variant="outline" asChild>
            <Link {...cancelLink}>Cancelar</Link>
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
