import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Link, Navigate, useNavigate } from '@tanstack/react-router'
import { useEffect } from 'react'
import { useAuth } from '@/auth/AuthContext'
import { resourceApi } from '@/api/client'
import { PageTitle } from '@/components/layout/PageTitle'
import { ResourceFormFields } from '@/components/resources/ResourceFormFields'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useResourceDef } from '@/hooks/useResourceDef'
import { buildDocumentoBody } from '@/lib/documentoParteLocalidade'
import { buildBody, useResourceFieldForm } from '@/lib/resourceForm'
import { resourceListTo, resourceShowTo } from '@/lib/resourcePaths'

export function ResourceNewPage() {
  const def = useResourceDef()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const api = resourceApi<Record<string, unknown>>(def.path)
  const { me } = useAuth()
  const form = useResourceFieldForm(def.fields)

  const empresaId = form.watch('empresa_id')

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
      form.setValue('data_competencia', new Date().toISOString().slice(0, 10))
    }
  }, [def.key, form])

  useEffect(() => {
    if (def.key !== 'documentos' || !empresaId) return
    const prestadorId = Number(empresaId)
    if (!Number.isFinite(prestadorId)) return

    const prestadorApi = resourceApi<{ municipio_id?: number }>('/api/prestadores')
    void prestadorApi.get(prestadorId).then((prestador) => {
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
  }, [def.key, empresaId, form])

  if (def.allowCreate === false) {
    return <Navigate {...resourceListTo(def)} replace />
  }

  const createMutation = useMutation({
    mutationFn: (values: Record<string, string>) => {
      const body =
        def.key === 'documentos'
          ? buildDocumentoBody(def.fields, values)
          : (buildBody(def.fields, values) as Record<string, unknown>)
      if (body.serie_numero_automatico === true) {
        body.numero_documento = 0
        body.serie = ''
      }
      return api.create(body)
    },
    onSuccess: (created) => {
      queryClient.invalidateQueries({ queryKey: [def.key] })
      const newId = Number(created.id)
      if (Number.isFinite(newId)) {
        navigate(resourceShowTo(def, newId))
      } else {
        navigate(resourceListTo(def))
      }
    },
  })

  return (
    <div className="space-y-4">
      <PageTitle>{`Novo — ${def.title}`}</PageTitle>
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link {...resourceListTo(def)}>← Voltar à lista</Link>
      </Button>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Cadastro</CardTitle>
        </CardHeader>
        <CardContent>
          <ResourceFormFields
            fields={def.fields}
            formSections={def.formSections}
            cncLookup={def.cncLookup}
            form={form}
            onSubmit={(values) => {
              createMutation.mutate(values)
            }}
            submitLabel="Criar"
            cancelLink={resourceListTo(def)}
            actionsAlign={def.key === 'documentos' ? 'end' : undefined}
          />
        </CardContent>
      </Card>
    </div>
  )
}
