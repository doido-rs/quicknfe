import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, Navigate } from '@tanstack/react-router'
import { useEffect } from 'react'
import { resourceApi } from '@/api/client'
import { ResourceFormFields } from '@/components/resources/ResourceFormFields'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useResourceDef, useResourceRecordId } from '@/hooks/useResourceDef'
import { buildBody, rowToForm, useResourceFieldForm } from '@/lib/resourceForm'
import { resourceListTo, resourceShowTo } from '@/lib/resourcePaths'

export function ResourceEditPage() {
  const def = useResourceDef()
  const recordId = useResourceRecordId()
  const queryClient = useQueryClient()
  const api = resourceApi<Record<string, unknown>>(def.path)
  const form = useResourceFieldForm(def.fields)

  const { data: row, isLoading, isError } = useQuery({
    queryKey: [def.key, 'detail', recordId],
    queryFn: () => api.get(recordId!),
    enabled: recordId != null,
  })

  useEffect(() => {
    if (row) {
      form.reset(rowToForm(row, def.fields))
    }
  }, [row, def.fields, form])

  const updateMutation = useMutation({
    mutationFn: (values: Record<string, string>) =>
      api.update(recordId!, buildBody(def.fields, values)),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [def.key] })
    },
  })

  if (recordId == null) {
    return <Navigate {...resourceListTo(def)} replace />
  }

  if (isLoading) {
    return <Skeleton className="h-48 w-full" />
  }

  if (isError || !row) {
    return <Navigate {...resourceListTo(def)} replace />
  }

  return (
    <div className="space-y-4">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link {...resourceShowTo(def, recordId)}>← Voltar ao registro</Link>
      </Button>
      <h1 className="text-2xl font-semibold tracking-tight">Editar — {def.title}</h1>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">ID {recordId}</CardTitle>
        </CardHeader>
        <CardContent>
          <ResourceFormFields
            fields={def.fields}
            formSections={def.formSections}
            cncLookup={def.cncLookup}
            form={form}
            onSubmit={(values) => {
              updateMutation.mutate(values)
            }}
            submitLabel="Salvar"
            cancelLink={resourceShowTo(def, recordId)}
          />
          {updateMutation.isSuccess && (
            <p className="mt-4 text-sm text-muted-foreground">Alterações salvas.</p>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
