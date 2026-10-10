import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useState, type FormEvent } from 'react'
import { Link, Navigate, useNavigate } from 'react-router-dom'
import { resourceApi } from '@/api/client'
import { ResourceFormFields } from '@/components/resources/ResourceFormFields'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useResourceDef } from '@/hooks/useResourceDef'
import { buildBody, emptyForm } from '@/lib/resourceForm'
import { resourceListPath, resourceShowPath } from '@/lib/resourcePaths'

export function ResourceNewPage() {
  const def = useResourceDef()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const api = resourceApi<Record<string, unknown>>(def.path)
  const [form, setForm] = useState(() => emptyForm(def.fields))

  if (def.allowCreate === false) {
    return <Navigate to={resourceListPath(def)} replace />
  }

  const createMutation = useMutation({
    mutationFn: () => api.create(buildBody(def.fields, form)),
    onSuccess: (created) => {
      queryClient.invalidateQueries({ queryKey: [def.key] })
      const newId = Number(created.id)
      if (Number.isFinite(newId)) {
        navigate(resourceShowPath(def, newId))
      } else {
        navigate(resourceListPath(def))
      }
    },
  })

  function onSubmit(e: FormEvent) {
    e.preventDefault()
    createMutation.mutate()
  }

  return (
    <div className="space-y-4">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link to={resourceListPath(def)}>← Voltar à lista</Link>
      </Button>
      <h1 className="text-2xl font-semibold tracking-tight">Novo — {def.title}</h1>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Cadastro</CardTitle>
        </CardHeader>
        <CardContent>
          <ResourceFormFields
            fields={def.fields}
            form={form}
            setForm={setForm}
            onSubmit={onSubmit}
            submitLabel="Criar"
            pending={createMutation.isPending}
            cancelTo={resourceListPath(def)}
          />
        </CardContent>
      </Card>
    </div>
  )
}
