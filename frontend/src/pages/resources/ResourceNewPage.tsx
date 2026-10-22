import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Navigate, useNavigate } from '@tanstack/react-router'
import { GuardedLink } from '@/components/navigation/GuardedLink'
import { useUnsavedFormGuard } from '@/hooks/useUnsavedFormGuard'
import { useEffect } from 'react'
import { useAuth } from '@/auth/AuthContext'
import { resourceApi } from '@/api/client'
import { PageTitle } from '@/components/layout/PageTitle'
import { DocumentoCreateFlowAlert } from '@/components/resources/DocumentoFormFlowAlert'
import { ResourceFormFields } from '@/components/resources/ResourceFormFields'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useResourceDef } from '@/hooks/useResourceDef'
import { buildDocumentoBody } from '@/lib/documentoParteLocalidade'
import { prepareDocumentoSubmitBody } from '@/lib/documentoSubmit'
import { runDocumentoFormValidation } from '@/lib/submitDocumentoForm'
import { todayApiDate } from '@/lib/dateFormat'
import { buildBody, useResourceFieldForm } from '@/lib/resourceForm'
import { resourceListTo, resourceShowTo } from '@/lib/resourcePaths'
import { toastFromError, toastSuccess } from '@/lib/toast'

export function ResourceNewPage() {
  const def = useResourceDef()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const api = resourceApi<Record<string, unknown>>(def.path)
  const { me } = useAuth()
  const form = useResourceFieldForm(def.fields)
  const empresaId = form.watch('empresa_id')
  const prestadorIdNum =
    def.key === 'documentos' && empresaId ? Number(empresaId) : Number.NaN
  const { data: documentoPrestador } = useQuery({
    queryKey: ['prestadores', 'detail', empresaId],
    queryFn: () =>
      resourceApi<{ regime_tributario?: string }>('/api/prestadores').get(prestadorIdNum),
    enabled: def.key === 'documentos' && Number.isFinite(prestadorIdNum),
  })

  useEffect(() => {
    if (def.key !== 'documentos' || !me?.prestador_ativo) return
    const current = form.getValues('empresa_id')
    if (!current) {
      form.setValue('empresa_id', String(me.prestador_ativo.id))
    }
  }, [def.key, me?.prestador_ativo, form])

  useEffect(() => {
    if (def.key !== 'documentos') return
    if (!form.getValues('data_competencia')) {
      form.setValue('data_competencia', todayApiDate())
    }
  }, [def.key, form])

  useEffect(() => {
    if (def.key !== 'documentos' || !empresaId) return
    const prestadorId = Number(empresaId)
    if (!Number.isFinite(prestadorId)) return

    let cancelled = false
    const prestadorApi = resourceApi<{ municipio_id?: number }>('/api/prestadores')
    void prestadorApi.get(prestadorId).then((prestador) => {
      if (cancelled) return
      if (form.getValues('empresa_id') !== String(prestadorId)) return
      const municipioId = prestador.municipio_id
      if (municipioId == null) return
      const asString = String(municipioId)
      if (!form.getValues('municipio_emissao_id')) {
        form.setValue('municipio_emissao_id', asString)
      }
      if (!form.getValues('municipio_prestacao_id')) {
        form.setValue('municipio_prestacao_id', asString)
      }
    })
    return () => {
      cancelled = true
    }
  }, [def.key, empresaId, form])

  useUnsavedFormGuard(form.formState.isDirty)

  const createMutation = useMutation({
    mutationFn: async (values: Record<string, string>) => {
      const body =
        def.key === 'documentos'
          ? buildDocumentoBody(def.fields, values)
          : (buildBody(def.fields, values) as Record<string, unknown>)
      prepareDocumentoSubmitBody(body, 'create')
      return api.create(body)
    },
    onSuccess: (created, _vars, _ctx) => {
      queryClient.invalidateQueries({ queryKey: [def.key] })
      toastSuccess('Registro criado com sucesso')
      form.reset(form.getValues())
      const newId = Number(created.id)
      if (Number.isFinite(newId)) {
        navigate(resourceShowTo(def, newId))
      } else {
        navigate(resourceListTo(def))
      }
    },
    onError: (err) => toastFromError(err, 'Não foi possível criar o registro'),
  })

  if (def.allowCreate === false) {
    return <Navigate {...resourceListTo(def)} replace />
  }

  return (
    <div className="space-y-4">
      <PageTitle>{`Novo — ${def.title}`}</PageTitle>
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <GuardedLink {...resourceListTo(def)}>← Voltar à lista</GuardedLink>
      </Button>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Cadastro</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {def.key === 'documentos' ? <DocumentoCreateFlowAlert /> : null}
          <ResourceFormFields
            fields={def.fields}
            formSections={def.formSections}
            cncLookup={def.cncLookup}
            form={form}
            onSubmit={(values) => {
              if (def.key === 'documentos' && !runDocumentoFormValidation(form)) return
              createMutation.mutate(values)
            }}
            submitLabel="Criar"
            cancelLink={resourceListTo(def)}
            actionsAlign={def.key === 'documentos' ? 'end' : undefined}
            documentoPrestadorRegime={documentoPrestador?.regime_tributario}
          />
        </CardContent>
      </Card>
    </div>
  )
}
