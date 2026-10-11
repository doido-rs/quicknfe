import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Link, Navigate, useNavigate } from '@tanstack/react-router'
import { useEffect } from 'react'
import { useAuth } from '@/auth/AuthContext'
import { resourceApi } from '@/api/client'
import { ResourceFormFields } from '@/components/resources/ResourceFormFields'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useResourceDef } from '@/hooks/useResourceDef'
import { buildBody, useResourceFieldForm } from '@/lib/resourceForm'
import { resourceListTo, resourceShowTo } from '@/lib/resourcePaths'

export function ResourceNewPage() {
  const def = useResourceDef()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const api = resourceApi<Record<string, unknown>>(def.path)
  const { me } = useAuth()
  const form = useResourceFieldForm(def.fields)

  useEffect(() => {
    if (def.key !== 'documentos' || !me?.prestador_ativo) return
    const current = form.getValues('empresa_id')
    if (!current) {
      form.setValue('empresa_id', String(me.prestador_ativo.id))
    }
  }, [def.key, me?.prestador_ativo, form])

  if (def.allowCreate === false) {
    return <Navigate {...resourceListTo(def)} replace />
  }

  const createMutation = useMutation({
    mutationFn: (values: Record<string, string>) => api.create(buildBody(def.fields, values)),
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
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link {...resourceListTo(def)}>← Voltar à lista</Link>
      </Button>
      <h1 className="text-2xl font-semibold tracking-tight">Novo — {def.title}</h1>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Cadastro</CardTitle>
        </CardHeader>
        <CardContent>
          <ResourceFormFields
            fields={def.fields}
            formSections={def.formSections}
            form={form}
            onSubmit={(values) => {
              createMutation.mutate(values)
            }}
            submitLabel="Criar"
            cancelLink={resourceListTo(def)}
          />
        </CardContent>
      </Card>
    </div>
  )
}
