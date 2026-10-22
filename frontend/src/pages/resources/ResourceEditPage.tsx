import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Navigate } from '@tanstack/react-router'
import { GuardedLink } from '@/components/navigation/GuardedLink'
import { useCallback, useEffect, useMemo, useRef } from 'react'
import { useRegisterUnsavedSave, useUnsavedFormGuard } from '@/hooks/useUnsavedFormGuard'
import { resourceApi } from '@/api/client'
import { PageTitle } from '@/components/layout/PageTitle'
import { DocumentoItensEditPanel } from '@/components/resources/DocumentoItensEditPanel'
import { DocumentoEditFlowAlert } from '@/components/resources/DocumentoFormFlowAlert'
import { ResourceFormFields } from '@/components/resources/ResourceFormFields'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useResourceDef, useResourceRecordId } from '@/hooks/useResourceDef'
import {
  applyItensToDocumentoBody,
  operacaoFormValueFromItens,
  sumDocumentoItensValor,
} from '@/lib/documentoItens'
import { normalizeMoneyToForm } from '@/lib/moneyFormat'
import { buildDocumentoBody, flattenDocumentoRowToForm } from '@/lib/documentoParteLocalidade'
import { parseDocumentoAmount } from '@/lib/documentoValorLiquido'
import { prepareDocumentoSubmitBody } from '@/lib/documentoSubmit'
import { runDocumentoFormValidation } from '@/lib/submitDocumentoForm'
import { buildBody, rowToForm, useResourceFieldForm } from '@/lib/resourceForm'
import { resourceListTo, resourceShowTo } from '@/lib/resourcePaths'
import { toastFromError, toastSuccess } from '@/lib/toast'
import { DOCUMENTO_EDIT_SECTION_TABS } from '@/lib/documentoEditTabs'
import { documentoItensNested } from '@/resources/definitions'

export function ResourceEditPage() {
  const def = useResourceDef()
  const recordId = useResourceRecordId()
  const queryClient = useQueryClient()
  const api = resourceApi<Record<string, unknown>>(def.path)
  const form = useResourceFieldForm(def.fields)
  const itensNested = useMemo(
    () => (def.key === 'documentos' ? documentoItensNested(def) : undefined),
    [def],
  )
  const prevEmpresaIdRef = useRef<string | undefined>(undefined)

  const { data: row, isLoading, isError } = useQuery({
    queryKey: [def.key, 'detail', recordId],
    queryFn: () => api.get(recordId!),
    enabled: recordId != null,
  })

  const itensListPath =
    def.key === 'documentos' && recordId != null && itensNested
      ? itensNested.path(recordId)
      : null
  const { data: itensPage } = useQuery({
    queryKey: [itensListPath, 'list'],
    queryFn: () => resourceApi<Record<string, unknown>>(itensListPath!).listPage({ pageSize: 50 }),
    enabled: itensListPath != null,
  })
  const documentoItensCount = itensPage?.items?.length

  useEffect(() => {
    if (def.key !== 'documentos' || !row) return
    const itens = itensPage?.items ?? []
    const sync = operacaoFormValueFromItens(itens, row.valor_total_servicos_produtos)
    if (!sync) return
    const current = normalizeMoneyToForm(form.getValues('valor_total_servicos_produtos'))
    if (current === sync.value) return
    form.setValue('valor_total_servicos_produtos', sync.value, {
      shouldDirty: sync.shouldDirty,
      shouldValidate: false,
    })
  }, [def.key, row, itensPage?.items, form])

  useEffect(() => {
    if (row) {
      const base = rowToForm(row, def.fields)
      form.reset(
        def.key === 'documentos' ? flattenDocumentoRowToForm(row, base) : base,
      )
      if (def.key === 'documentos') {
        prevEmpresaIdRef.current = undefined
      }
    }
  }, [row, def.fields, def.key, form])

  const {
    handleSubmit,
    formState: { isDirty },
    watch,
  } = form
  const itemRelationScope =
    def.key === 'documentos'
      ? { empresa_id: watch('empresa_id') ?? '' }
      : undefined
  const editEmpresaId = def.key === 'documentos' ? watch('empresa_id') : undefined
  const prestadorIdNum =
    def.key === 'documentos' && editEmpresaId
      ? Number(editEmpresaId)
      : Number.NaN
  const { data: documentoPrestador } = useQuery({
    queryKey: ['prestadores', 'detail', editEmpresaId],
    queryFn: () =>
      resourceApi<{ regime_tributario?: string }>('/api/prestadores').get(prestadorIdNum),
    enabled: def.key === 'documentos' && Number.isFinite(prestadorIdNum),
  })

  useEffect(() => {
    if (def.key !== 'documentos' || !editEmpresaId) return
    const prev = prevEmpresaIdRef.current
    if (prev === undefined) {
      prevEmpresaIdRef.current = editEmpresaId
      return
    }
    if (prev === editEmpresaId) return
    prevEmpresaIdRef.current = editEmpresaId

    form.setValue('cliente_id', '', { shouldDirty: true })
    form.setValue('intermediario_id', '', { shouldDirty: true })

    let cancelled = false
    const prestadorId = Number(editEmpresaId)
    if (!Number.isFinite(prestadorId)) return
    const prestadorApi = resourceApi<{ municipio_id?: number }>('/api/prestadores')
    void prestadorApi.get(prestadorId).then((prestador) => {
      if (cancelled || form.getValues('empresa_id') !== editEmpresaId) return
      const municipioId = prestador.municipio_id
      if (municipioId == null) return
      const asString = String(municipioId)
      form.setValue('municipio_emissao_id', asString, { shouldDirty: true })
      form.setValue('municipio_prestacao_id', asString, { shouldDirty: true })
    })
    return () => {
      cancelled = true
    }
  }, [def.key, editEmpresaId, form])

  useUnsavedFormGuard(isDirty)

  const updateMutation = useMutation({
    mutationFn: async (values: Record<string, string>) => {
      const body =
        def.key === 'documentos'
          ? buildDocumentoBody(def.fields, values)
          : (buildBody(def.fields, values) as Record<string, unknown>)
      prepareDocumentoSubmitBody(body, 'update')

      let syncedFromItens = false
      if (def.key === 'documentos' && recordId != null && itensNested) {
        const itensPath = itensNested.path(recordId)
        const cached = queryClient.getQueryData<{ items: Record<string, unknown>[] }>([
          itensPath,
          'list',
        ])
        let itens = cached?.items
        if (!itens) {
          const page = await resourceApi<Record<string, unknown>>(itensPath).listPage({
            pageSize: 50,
          })
          itens = page.items
        }
        const manualOperacao = parseDocumentoAmount(values.valor_total_servicos_produtos)
        const sumItens = sumDocumentoItensValor(itens ?? [])
        applyItensToDocumentoBody(body, values, itens ?? [])
        syncedFromItens =
          sumItens > 0 && manualOperacao !== sumItens && body.valor_total_servicos_produtos === sumItens
      }

      const updated = await api.update(recordId!, body)
      return { updated, syncedFromItens, values }
    },
    onSuccess: ({ syncedFromItens, values }) => {
      queryClient.invalidateQueries({ queryKey: [def.key] })
      form.reset(values)
      if (syncedFromItens) {
        toastSuccess('Alterações salvas. Valor da operação atualizado conforme itens.')
      } else {
        toastSuccess('Alterações salvas')
      }
    },
    onError: (err) => toastFromError(err, 'Não foi possível salvar'),
  })

  const saveFromDialog = useCallback((): Promise<boolean> => {
    return new Promise((resolve) => {
      if (def.key === 'documentos' && !runDocumentoFormValidation(form)) {
        resolve(false)
        return
      }
      void handleSubmit(
        async (values) => {
          try {
            await updateMutation.mutateAsync(values)
            resolve(true)
          } catch {
            resolve(false)
          }
        },
        () => resolve(false),
      )()
    })
  }, [def.key, form, handleSubmit, updateMutation])

  useRegisterUnsavedSave(saveFromDialog)

  if (recordId == null) {
    return <Navigate {...resourceListTo(def)} replace />
  }

  if (isLoading) {
    return (
      <>
        <PageTitle>{`Editar — ${def.title}`}</PageTitle>
        <Skeleton className="h-48 w-full" />
      </>
    )
  }

  if (isError || !row) {
    return <Navigate {...resourceListTo(def)} replace />
  }

  return (
    <div className="space-y-4">
      <PageTitle>{`Editar — ${def.title}`}</PageTitle>
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <GuardedLink {...resourceShowTo(def, recordId)}>← Voltar ao registro</GuardedLink>
      </Button>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">ID {recordId}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {def.key === 'documentos' ? (
            <DocumentoEditFlowAlert itensCount={documentoItensCount} />
          ) : null}
          <ResourceFormFields
            fields={def.fields}
            formSections={def.formSections}
            cncLookup={def.cncLookup}
            form={form}
            sectionTabs={def.key === 'documentos' ? DOCUMENTO_EDIT_SECTION_TABS : undefined}
            itensTab={
              def.key === 'documentos' && itensNested
                ? {
                    label: 'Itens',
                    count: documentoItensCount,
                    content: (
                      <DocumentoItensEditPanel
                        parentId={recordId}
                        nested={itensNested}
                        relationScopeValues={itemRelationScope}
                      />
                    ),
                  }
                : undefined
            }
            onSubmit={(values) => {
              if (def.key === 'documentos' && !runDocumentoFormValidation(form)) return
              updateMutation.mutate(values)
            }}
            submitLabel="Salvar cabeçalho"
            cancelLink={resourceShowTo(def, recordId)}
            actionsAlign={def.key === 'documentos' ? 'end' : undefined}
            documentoPrestadorRegime={documentoPrestador?.regime_tributario}
          />
          {updateMutation.isSuccess && (
            <p className="mt-4 text-sm text-muted-foreground">Alterações salvas.</p>
          )}
        </CardContent>
      </Card>

    </div>
  )
}
