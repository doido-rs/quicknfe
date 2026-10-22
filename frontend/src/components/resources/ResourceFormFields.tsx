import { GuardedLink } from '@/components/navigation/GuardedLink'
import { useUnsavedFormGuard } from '@/hooks/useUnsavedFormGuard'
import { computeDocumentoValorLiquido } from '@/lib/documentoValorLiquido'
import { useEffect } from 'react'
import type { UseFormReturn } from 'react-hook-form'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { CncConsultaField } from '@/components/resources/CncConsultaField'
import { ResourceFieldInput } from '@/components/resources/ResourceFieldInput'
import type { ResourceRouteLink } from '@/lib/resourcePaths'
import { fieldsByKeys } from '@/lib/resourceForm'
import { ParteLocalidadeFields } from '@/components/resources/ParteLocalidadeFields'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { buildDocumentoEditTabOrder, DOCUMENTO_EDIT_ITENS_TAB } from '@/lib/documentoEditTabs'
import type { CncLookupDef, FieldDef, FormSectionDef } from '@/resources/definitions'
import type { ReactNode } from 'react'

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
  columns?: 1 | 2 | 3
  readOnly?: boolean
  relationDisplayLabels?: Record<string, string>
  displayRow?: Record<string, unknown>
}) {
  const gridClass =
    columns === 1
      ? 'grid gap-4'
      : columns === 3
        ? 'grid gap-4 md:grid-cols-2 lg:grid-cols-3'
        : 'grid gap-4 md:grid-cols-2'

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
  compact,
}: {
  section: Extract<FormSectionDef, { type: 'identificacao-nota' }>
  allFields: FieldDef[]
  form: UseFormReturn<Record<string, string>>
  sectionIndex: number
  readOnly?: boolean
  relationDisplayLabels?: Record<string, string>
  displayRow?: Record<string, unknown>
  compact?: boolean
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
      className={compact ? 'rounded-lg border bg-card p-3 md:p-4' : 'rounded-lg border bg-card p-4 md:p-6'}
      aria-labelledby={`form-section-${sectionIndex}-identificacao`}
    >
      <h2
        id={`form-section-${sectionIndex}-identificacao`}
        className={compact ? 'mb-3 text-base font-semibold tracking-tight' : 'mb-4 text-lg font-semibold tracking-tight'}
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
                  form.setValue(section.automaticoFieldKey, e.target.checked ? 'false' : 'true', {
                    shouldDirty: true,
                  })
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
  compact,
}: {
  section: FormSectionDef
  allFields: FieldDef[]
  form: UseFormReturn<Record<string, string>>
  sectionIndex: number
  readOnly?: boolean
  relationDisplayLabels?: Record<string, string>
  displayRow?: Record<string, unknown>
  compact?: boolean
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
        compact={compact}
      />
    )
  }

  if (section.type === 'municipios-nota') {
    const emissao = fieldsByKeys(allFields, [section.emissaoFieldKey])
    const prestacao = fieldsByKeys(allFields, [section.prestacaoFieldKey])
    return (
      <section
        className={compact ? 'rounded-lg border bg-card p-3 md:p-4' : 'rounded-lg border bg-card p-4 md:p-6'}
        aria-labelledby={`form-section-${sectionIndex}-municipios`}
      >
        <h2
          id={`form-section-${sectionIndex}-municipios`}
          className={compact ? 'mb-3 text-base font-semibold tracking-tight' : 'mb-4 text-lg font-semibold tracking-tight'}
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

  const issqnSuspensa = form.watch('issqn_exigibilidade_suspensa') === 'true'
  const sectionFieldKeys =
    section.type === 'fields' && section.fieldKeys.includes('issqn_exigibilidade_suspensa')
      ? section.fieldKeys.filter((key) => {
          if (key === 'issqn_tp_suspensao' || key === 'issqn_n_processo_suspensao') {
            return issqnSuspensa
          }
          return true
        })
      : section.fieldKeys

  const sectionFields = fieldsByKeys(allFields, sectionFieldKeys).map((f) => {
    if (f.key === 'issqn_n_processo_suspensao' && issqnSuspensa) {
      return { ...f, required: true }
    }
    return f
  })

  return (
    <section
      className={compact ? 'rounded-lg border bg-card p-3 md:p-4' : 'rounded-lg border bg-card p-4 md:p-6'}
      aria-labelledby={`form-section-${sectionIndex}`}
    >
      <h2
        id={`form-section-${sectionIndex}`}
        className={compact ? 'mb-3 text-base font-semibold tracking-tight' : 'mb-4 text-lg font-semibold tracking-tight'}
      >
        {section.title}
      </h2>
      <SectionFields
        fields={sectionFields}
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
  sectionTabs,
  itensTab,
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
  /** Agrupa seções em abas (edição de nota fiscal). */
  sectionTabs?: ReadonlyArray<{
    value: string
    label: string
    sectionIndexes: readonly number[]
  }>
  /** Aba extra de itens (conteúdo fora do fluxo de submit do cabeçalho). */
  itensTab?: { label: string; count?: number; content: ReactNode }
}) {
  const documentoTabOrder =
    sectionTabs?.length && itensTab
      ? buildDocumentoEditTabOrder(true, itensTab.label)
      : sectionTabs?.map((tab) => ({ kind: 'section' as const, ...tab })) ?? []
  const {
    handleSubmit,
    formState: { isSubmitting, isDirty },
  } = form

  const hasValorLiquidoField = fields.some((f) => f.key === 'valor_liquido')
  const watchOperacao = form.watch('valor_total_servicos_produtos')
  const watchDescIncond = form.watch('desconto_incondicionado')
  const watchDescCond = form.watch('desconto_condicionado')

  useEffect(() => {
    if (!hasValorLiquidoField) return
    const next = computeDocumentoValorLiquido({
      valor_total_servicos_produtos: watchOperacao ?? '',
      desconto_incondicionado: watchDescIncond ?? '',
      desconto_condicionado: watchDescCond ?? '',
    })
    const current = form.getValues('valor_liquido')
    if (current !== String(next)) {
      form.setValue('valor_liquido', next.toFixed(2), { shouldDirty: false, shouldValidate: false })
    }
  }, [hasValorLiquidoField, watchOperacao, watchDescIncond, watchDescCond, form])

  useUnsavedFormGuard(!readOnly && isDirty)

  const cancelButton = cancelLink ? (
    <Button type="button" variant="outline" asChild>
      <GuardedLink {...cancelLink}>Cancelar</GuardedLink>
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

  const renderSection = (index: number, compact?: boolean) => {
    const section = formSections?.[index]
    if (!section) return null
    return (
      <FormSectionBlock
        key={`${section.type}-${index}`}
        section={section}
        allFields={fields}
        form={form}
        sectionIndex={index}
        readOnly={readOnly}
        relationDisplayLabels={relationDisplayLabels}
        displayRow={displayRow}
        compact={compact}
      />
    )
  }

  const stackedBody = formSections?.length ? (
    <>
      {cncBlock && <div className="grid gap-4">{cncBlock}</div>}
      {formSections.map((section, index) => renderSection(index))}
    </>
  ) : null

  const tabbedBody =
    formSections?.length && sectionTabs?.length ? (
      <>
        {cncBlock && <div className="mb-4 grid gap-4">{cncBlock}</div>}
        <Tabs defaultValue={sectionTabs[0]?.value} className="w-full">
          <TabsList
            aria-label="Grupos da nota fiscal"
            className="mb-4 h-auto flex-wrap justify-start gap-1"
          >
            {documentoTabOrder.map((entry) => (
              <TabsTrigger
                key={entry.value}
                value={entry.value}
                className="text-xs sm:text-sm"
              >
                {entry.label}
                {entry.kind === 'itens' && itensTab?.count != null
                  ? ` (${itensTab.count})`
                  : ''}
              </TabsTrigger>
            ))}
          </TabsList>
          {documentoTabOrder.map((entry) =>
            entry.kind === 'itens' ? (
              <TabsContent key={entry.value} value={DOCUMENTO_EDIT_ITENS_TAB} className="mt-0">
                {itensTab?.content}
              </TabsContent>
            ) : (
              <TabsContent key={entry.value} value={entry.value} className="mt-0 space-y-4">
                {entry.sectionIndexes.map((index) => renderSection(index, true))}
              </TabsContent>
            ),
          )}
        </Tabs>
      </>
    ) : null

  const body = tabbedBody ?? stackedBody

  if (formSections?.length) {
    if (readOnly) {
      return <div className="space-y-8">{stackedBody}</div>
    }
    return (
      <form onSubmit={handleSubmit(onSubmit!)} className="space-y-6">
        {body}
        <div
          className={
            actionsAlign === 'end'
              ? 'sticky bottom-0 z-10 -mx-1 border-t bg-card/95 px-1 py-3 backdrop-blur supports-[backdrop-filter]:bg-card/80'
              : undefined
          }
        >
          {actions}
        </div>
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
