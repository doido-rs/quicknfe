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
  readOnly,
  relationDisplayLabels,
  displayRow,
}: {
  fields: FieldDef[]
  form: UseFormReturn<Record<string, string>>
  idPrefix: string
  columns?: 1 | 2
  readOnly?: boolean
  relationDisplayLabels?: Record<string, string>
  displayRow?: Record<string, unknown>
}) {
  const gridClass = columns === 1 ? 'grid gap-4' : 'grid gap-4 md:grid-cols-2'

  return (
    <div className={gridClass}>
      {fields.map((field) => (
        <ResourceFieldInput
          key={field.key}
          field={field}
          form={form}
          idPrefix={idPrefix}
          readOnly={readOnly}
          relationDisplayLabels={relationDisplayLabels}
          displayRow={displayRow}
        />
      ))}
    </div>
  )
}

function IdentificacaoNotaSection({
  section,
  allFields,
  form,
  sectionIndex,
  readOnly,
  relationDisplayLabels,
  displayRow,
}: {
  section: Extract<FormSectionDef, { type: 'identificacao-nota' }>
  allFields: FieldDef[]
  form: UseFormReturn<Record<string, string>>
  sectionIndex: number
  readOnly?: boolean
  relationDisplayLabels?: Record<string, string>
  displayRow?: Record<string, unknown>
}) {
  const automatico = form.watch(section.automaticoFieldKey) === 'true'
  const informarManual = readOnly || !automatico
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
              readOnly={readOnly}
              relationDisplayLabels={relationDisplayLabels}
              displayRow={displayRow}
            />
          ))}
        </div>
        <div className="grid gap-4 md:col-span-3 sm:grid-cols-2">
          {!readOnly && (
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
          )}
          {informarManual &&
            [...serie, ...numero].map((field) => (
              <ResourceFieldInput
                key={field.key}
                field={field}
                form={form}
                idPrefix={`s${sectionIndex}-num`}
                readOnly={readOnly}
                relationDisplayLabels={relationDisplayLabels}
                displayRow={displayRow}
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
  readOnly,
  relationDisplayLabels,
  displayRow,
}: {
  section: FormSectionDef
  allFields: FieldDef[]
  form: UseFormReturn<Record<string, string>>
  sectionIndex: number
  readOnly?: boolean
  relationDisplayLabels?: Record<string, string>
  displayRow?: Record<string, unknown>
}) {
  if (section.type === 'identificacao-nota') {
    return (
      <IdentificacaoNotaSection
        section={section}
        allFields={allFields}
        form={form}
        sectionIndex={sectionIndex}
        readOnly={readOnly}
        relationDisplayLabels={relationDisplayLabels}
        displayRow={displayRow}
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
              readOnly={readOnly}
              relationDisplayLabels={relationDisplayLabels}
              displayRow={displayRow}
            />
          ))}
          {prestacao.map((field) => (
            <ResourceFieldInput
              key={field.key}
              field={field}
              form={form}
              idPrefix={`s${sectionIndex}-mun-pr`}
              readOnly={readOnly}
              relationDisplayLabels={relationDisplayLabels}
              displayRow={displayRow}
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
            readOnly={readOnly}
            relationDisplayLabels={relationDisplayLabels}
            displayRow={displayRow}
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
              readOnly={readOnly}
              relationDisplayLabels={relationDisplayLabels}
              displayRow={displayRow}
            />
          ) : (
            <SectionFields
              fields={fieldsByKeys(allFields, section.partesFieldKeys ?? [])}
              form={form}
              idPrefix={`s${sectionIndex}-partes`}
              columns={1}
              readOnly={readOnly}
              relationDisplayLabels={relationDisplayLabels}
              displayRow={displayRow}
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
        readOnly={readOnly}
        relationDisplayLabels={relationDisplayLabels}
        displayRow={displayRow}
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
  actionsAlign,
  readOnly,
  relationDisplayLabels,
  displayRow,
}: {
  fields: FieldDef[]
  formSections?: FormSectionDef[]
  form: UseFormReturn<Record<string, string>>
  cncLookup?: CncLookupDef
  onSubmit?: (values: Record<string, string>) => void | Promise<void>
  submitLabel?: string
  cancelLink?: ResourceRouteLink
  onCancel?: () => void
  /** Alinha Criar/Cancelar à direita (ex.: notas fiscais). */
  actionsAlign?: 'end'
  readOnly?: boolean
  relationDisplayLabels?: Record<string, string>
  displayRow?: Record<string, unknown>
}) {
  const {
    handleSubmit,
    formState: { isSubmitting },
  } = form

  const cancelButton = cancelLink ? (
    <Button type="button" variant="outline" asChild>
      <Link {...cancelLink}>Cancelar</Link>
    </Button>
  ) : onCancel ? (
    <Button type="button" variant="outline" onClick={onCancel}>
      Cancelar
    </Button>
  ) : null

  const actions = (
    <div
      className={
        actionsAlign === 'end' ? 'flex flex-wrap justify-end gap-2' : 'flex flex-wrap gap-2'
      }
    >
      {actionsAlign === 'end' ? cancelButton : null}
      <Button type="submit" disabled={isSubmitting}>
        {submitLabel}
      </Button>
      {actionsAlign === 'end' ? null : cancelButton}
    </div>
  )

  const cncBlock =
    cncLookup && !readOnly ? <CncConsultaField lookup={cncLookup} form={form} /> : null

  const body = formSections?.length ? (
    <>
      {cncBlock && <div className="grid gap-4">{cncBlock}</div>}
      {formSections.map((section, index) => (
        <FormSectionBlock
          key={`${section.type}-${index}`}
          section={section}
          allFields={fields}
          form={form}
          sectionIndex={index}
          readOnly={readOnly}
          relationDisplayLabels={relationDisplayLabels}
          displayRow={displayRow}
        />
      ))}
    </>
  ) : null

  if (formSections?.length) {
    if (readOnly) {
      return <div className="space-y-8">{body}</div>
    }
    return (
      <form onSubmit={handleSubmit(onSubmit!)} className="space-y-8">
        {body}
        {actions}
      </form>
    )
  }

  return (
    <form onSubmit={handleSubmit(onSubmit!)} className="grid gap-4 md:grid-cols-2">
      {cncBlock}
      {fields.map((field) => (
        <ResourceFieldInput key={field.key} field={field} form={form} />
      ))}
      <div className="flex flex-wrap gap-2 md:col-span-2">{actions}</div>
    </form>
  )
}
