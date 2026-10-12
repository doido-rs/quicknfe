import { Link } from '@tanstack/react-router'
import type { UseFormReturn } from 'react-hook-form'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { CncConsultaField } from '@/components/resources/CncConsultaField'
import { ResourceFieldInput } from '@/components/resources/ResourceFieldInput'
import type { ResourceRouteLink } from '@/lib/resourcePaths'
import { fieldsByKeys } from '@/lib/resourceForm'
import { ParteLocalidadeFields } from '@/components/resources/ParteLocalidadeFields'
import type { CncLookupDef, FieldDef, FormSectionDef } from '@/resources/definitions'

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

function IdentificacaoNotaSection({
  section,
  allFields,
  form,
  sectionIndex,
}: {
  section: Extract<FormSectionDef, { type: 'identificacao-nota' }>
  allFields: FieldDef[]
  form: UseFormReturn<Record<string, string>>
  sectionIndex: number
}) {
  const automatico = form.watch(section.automaticoFieldKey) === 'true'
  const informarManual = !automatico
  const competencia = fieldsByKeys(allFields, [section.competenciaFieldKey])
  const serie = fieldsByKeys(allFields, [section.serieFieldKey]).map((f) => ({
    ...f,
    required: informarManual && f.required,
  }))
  const numero = fieldsByKeys(allFields, [section.numeroFieldKey]).map((f) => ({
    ...f,
    required: informarManual && f.required,
  }))
  const manualCheckboxId = `s${sectionIndex}-informar-serie-numero`

  return (
    <section
      className="rounded-lg border bg-card p-4 md:p-6"
      aria-labelledby={`form-section-${sectionIndex}-identificacao`}
    >
      <h2
        id={`form-section-${sectionIndex}-identificacao`}
        className="mb-4 text-lg font-semibold tracking-tight"
      >
        {section.title}
      </h2>
      <div className="grid gap-6 md:grid-cols-5 md:items-start">
        <div className="md:col-span-2">
          {competencia.map((field) => (
            <ResourceFieldInput
              key={field.key}
              field={field}
              form={form}
              idPrefix={`s${sectionIndex}-comp`}
            />
          ))}
        </div>
        <div className="grid gap-4 md:col-span-3 sm:grid-cols-2">
          <div className="flex items-center gap-2 sm:col-span-2">
            <input
              id={manualCheckboxId}
              type="checkbox"
              className="size-4 shrink-0 rounded border border-input"
              checked={informarManual}
              onChange={(e) =>
                form.setValue(section.automaticoFieldKey, e.target.checked ? 'false' : 'true')
              }
            />
            <Label htmlFor={manualCheckboxId} className="font-normal leading-snug">
              {section.manualInformLabel}
            </Label>
          </div>
          {informarManual &&
            [...serie, ...numero].map((field) => (
              <ResourceFieldInput
                key={field.key}
                field={field}
                form={form}
                idPrefix={`s${sectionIndex}-num`}
              />
            ))}
        </div>
      </div>
    </section>
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
  if (section.type === 'identificacao-nota') {
    return (
      <IdentificacaoNotaSection
        section={section}
        allFields={allFields}
        form={form}
        sectionIndex={sectionIndex}
      />
    )
  }

  if (section.type === 'municipios-nota') {
    const emissao = fieldsByKeys(allFields, [section.emissaoFieldKey])
    const prestacao = fieldsByKeys(allFields, [section.prestacaoFieldKey])
    return (
      <section
        className="rounded-lg border bg-card p-4 md:p-6"
        aria-labelledby={`form-section-${sectionIndex}-municipios`}
      >
        <h2
          id={`form-section-${sectionIndex}-municipios`}
          className="mb-4 text-lg font-semibold tracking-tight"
        >
          {section.title}
        </h2>
        <div className="grid gap-4 md:grid-cols-2">
          {emissao.map((field) => (
            <ResourceFieldInput
              key={field.key}
              field={field}
              form={form}
              idPrefix={`s${sectionIndex}-mun-em`}
            />
          ))}
          {prestacao.map((field) => (
            <ResourceFieldInput
              key={field.key}
              field={field}
              form={form}
              idPrefix={`s${sectionIndex}-mun-pr`}
            />
          ))}
        </div>
      </section>
    )
  }

  if (section.type === 'prestador-partes-row') {
    const prestadorFields = fieldsByKeys(allFields, [section.prestadorFieldKey])
    return (
      <div className="grid gap-6 lg:grid-cols-2 lg:items-stretch">
        <section
          className="flex h-full flex-col rounded-lg border bg-card p-4 md:p-6"
          aria-labelledby={`form-section-${sectionIndex}-prestador`}
        >
          <h2
            id={`form-section-${sectionIndex}-prestador`}
            className="mb-4 text-lg font-semibold tracking-tight"
          >
            {section.prestadorTitle}
          </h2>
          <SectionFields
            fields={prestadorFields}
            form={form}
            idPrefix={`s${sectionIndex}-prestador`}
            columns={1}
          />
        </section>
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
          {section.partesLocalidade ? (
            <ParteLocalidadeFields
              allFields={allFields}
              form={form}
              sectionIndex={sectionIndex}
            />
          ) : (
            <SectionFields
              fields={fieldsByKeys(allFields, section.partesFieldKeys ?? [])}
              form={form}
              idPrefix={`s${sectionIndex}-partes`}
              columns={1}
            />
          )}
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
  cncLookup,
  onSubmit,
  submitLabel,
  cancelLink,
  onCancel,
}: {
  fields: FieldDef[]
  formSections?: FormSectionDef[]
  form: UseFormReturn<Record<string, string>>
  cncLookup?: CncLookupDef
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

  const cncBlock = cncLookup ? <CncConsultaField lookup={cncLookup} form={form} /> : null

  if (formSections?.length) {
    return (
      <form onSubmit={handleSubmit(onSubmit)} className="space-y-8">
        {cncBlock && <div className="grid gap-4">{cncBlock}</div>}
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
      {cncBlock}
      {fields.map((field) => (
        <ResourceFieldInput key={field.key} field={field} form={form} />
      ))}
      <div className="flex flex-wrap gap-2 md:col-span-2">{actions}</div>
    </form>
  )
}
