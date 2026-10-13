import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, Navigate } from '@tanstack/react-router'
import { useEffect } from 'react'
import { resourceApi } from '@/api/client'
import { PageTitle } from '@/components/layout/PageTitle'
import { ResourceFormFields } from '@/components/resources/ResourceFormFields'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useResourceDef, useResourceRecordId } from '@/hooks/useResourceDef'
import { buildDocumentoBody, flattenDocumentoRowToForm } from '@/lib/documentoParteLocalidade'
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
      const base = rowToForm(row, def.fields)
      form.reset(
        def.key === 'documentos' ? flattenDocumentoRowToForm(row, base) : base,
      )
    }
  }, [row, def.fields, form])

  const updateMutation = useMutation({
    mutationFn: (values: Record<string, string>) => {
      const body =
        def.key === 'documentos'
          ? buildDocumentoBody(def.fields, values)
          : (buildBody(def.fields, values) as Record<string, unknown>)
      if (body.serie_numero_automatico === true) {
        body.numero_documento = 0
        body.serie = ''
      }
      return api.update(recordId!, body)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [def.key] })
    },
  })

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
        <Link {...resourceShowTo(def, recordId)}>← Voltar ao registro</Link>
      </Button>

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
            actionsAlign={def.key === 'documentos' ? 'end' : undefined}
          />
          {updateMutation.isSuccess && (
            <p className="mt-4 text-sm text-muted-foreground">Alterações salvas.</p>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
