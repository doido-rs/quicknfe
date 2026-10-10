import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect, useState, type FormEvent } from 'react'
import { Link, Navigate, useParams } from 'react-router-dom'
import { resourceApi } from '@/api/client'
import { ResourceFormFields } from '@/components/resources/ResourceFormFields'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useResourceDef } from '@/hooks/useResourceDef'
import { buildBody, emptyForm, rowToForm } from '@/lib/resourceForm'
import { resourceListPath, resourceShowPath } from '@/lib/resourcePaths'

export function ResourceEditPage() {
  const def = useResourceDef()
  const { id } = useParams<{ id: string }>()
  const queryClient = useQueryClient()
  const recordId = Number(id)
  const api = resourceApi<Record<string, unknown>>(def.path)
  const [form, setForm] = useState(() => emptyForm(def.fields))

  const { data: row, isLoading, isError } = useQuery({
    queryKey: [def.key, 'detail', recordId],
    queryFn: () => api.get(recordId),
    enabled: Number.isFinite(recordId),
  })

  useEffect(() => {
    if (row) {
      setForm(rowToForm(row, def.fields))
    }
  }, [row, def.fields])

  const updateMutation = useMutation({
    mutationFn: () => api.update(recordId, buildBody(def.fields, form)),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [def.key] })
    },
  })

  if (!Number.isFinite(recordId)) {
    return <Navigate to={resourceListPath(def)} replace />
  }

  if (isLoading) {
    return <Skeleton className="h-48 w-full" />
  }

  if (isError || !row) {
    return <Navigate to={resourceListPath(def)} replace />
  }

  function onSubmit(e: FormEvent) {
    e.preventDefault()
    updateMutation.mutate()
  }

  return (
    <div className="space-y-4">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link to={resourceShowPath(def, recordId)}>← Voltar ao registro</Link>
      </Button>
      <h1 className="text-2xl font-semibold tracking-tight">Editar — {def.title}</h1>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">ID {recordId}</CardTitle>
        </CardHeader>
        <CardContent>
          <ResourceFormFields
            fields={def.fields}
            form={form}
            setForm={setForm}
            onSubmit={onSubmit}
            submitLabel="Salvar"
            pending={updateMutation.isPending}
            cancelTo={resourceShowPath(def, recordId)}
          />
          {updateMutation.isSuccess && (
            <p className="mt-4 text-sm text-muted-foreground">Alterações salvas.</p>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
