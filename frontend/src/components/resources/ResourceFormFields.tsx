import { Link } from '@tanstack/react-router'
import type { UseFormReturn } from 'react-hook-form'
import { Button } from '@/components/ui/button'
import { ResourceFieldInput } from '@/components/resources/ResourceFieldInput'
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
    handleSubmit,
    formState: { isSubmitting },
  } = form

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="grid gap-4 md:grid-cols-2">
      {fields.map((field) => (
        <ResourceFieldInput key={field.key} field={field} form={form} />
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
