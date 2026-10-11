import { Link } from '@tanstack/react-router'
import type { UseFormReturn } from 'react-hook-form'
import { Button } from '@/components/ui/button'
import { PrestadorSessionPanel } from '@/components/resources/PrestadorSessionPanel'
import { ResourceFieldInput } from '@/components/resources/ResourceFieldInput'
import type { ResourceRouteLink } from '@/lib/resourcePaths'
import { fieldsByKeys } from '@/lib/resourceForm'
import type { FieldDef, FormSectionDef } from '@/resources/definitions'

function SectionFields({
  fields,
  form,
  idPrefix,
  columns = 2,
}: {
  fields: FieldDef[]
  form: UseFormReturn<Record<string, string>>
  idPrefix: string
  columns?: 1 | 2
}) {
  const gridClass = columns === 1 ? 'grid gap-4' : 'grid gap-4 md:grid-cols-2'

  return (
    <div className={gridClass}>
      {fields.map((field) => (
        <ResourceFieldInput key={field.key} field={field} form={form} idPrefix={idPrefix} />
      ))}
    </div>
  )
}

function FormSectionBlock({
  section,
  allFields,
  form,
  sectionIndex,
}: {
  section: FormSectionDef
  allFields: FieldDef[]
  form: UseFormReturn<Record<string, string>>
  sectionIndex: number
}) {
  if (section.type === 'prestador-partes-row') {
    return (
      <div className="grid gap-6 lg:grid-cols-2 lg:items-stretch">
        <PrestadorSessionPanel title={section.prestadorTitle} className="h-full" />
        <section
          className="flex h-full flex-col rounded-lg border bg-card p-4 md:p-6"
          aria-labelledby={`form-section-${sectionIndex}-partes`}
        >
          <h2
            id={`form-section-${sectionIndex}-partes`}
            className="mb-4 text-lg font-semibold tracking-tight"
          >
            {section.partesTitle}
          </h2>
          <SectionFields
            fields={fieldsByKeys(allFields, section.partesFieldKeys)}
            form={form}
            idPrefix={`s${sectionIndex}-partes`}
            columns={1}
          />
        </section>
      </div>
    )
  }

  return (
    <section
      className="rounded-lg border bg-card p-4 md:p-6"
      aria-labelledby={`form-section-${sectionIndex}`}
    >
      <h2 id={`form-section-${sectionIndex}`} className="mb-4 text-lg font-semibold tracking-tight">
        {section.title}
      </h2>
      <SectionFields
        fields={fieldsByKeys(allFields, section.fieldKeys)}
        form={form}
        idPrefix={`s${sectionIndex}`}
        columns={section.columns ?? 2}
      />
    </section>
  )
}

export function ResourceFormFields({
  fields,
  formSections,
  form,
  onSubmit,
  submitLabel,
  cancelLink,
  onCancel,
}: {
  fields: FieldDef[]
  formSections?: FormSectionDef[]
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

  const actions = (
    <div className="flex flex-wrap gap-2">
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
  )

  if (formSections?.length) {
    return (
      <form onSubmit={handleSubmit(onSubmit)} className="space-y-8">
        {formSections.map((section, index) => (
          <FormSectionBlock
            key={`${section.type}-${index}`}
            section={section}
            allFields={fields}
            form={form}
            sectionIndex={index}
          />
        ))}
        {actions}
      </form>
    )
  }

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="grid gap-4 md:grid-cols-2">
      {fields.map((field) => (
        <ResourceFieldInput key={field.key} field={field} form={form} />
      ))}
      <div className="flex flex-wrap gap-2 md:col-span-2">{actions}</div>
    </form>
  )
}
