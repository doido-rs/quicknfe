import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, Navigate, useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { resourceApi } from '@/api/client'
import { DocumentoPartesShowCard } from '@/components/resources/DocumentoPartesShowCard'
import { DocumentoXmlDownloadButton } from '@/components/resources/DocumentoXmlDownloadButton'
import { DeleteConfirmDialog } from '@/components/resources/DeleteConfirmDialog'
import { NestedResourcePanel } from '@/components/resources/NestedResourcePanel'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useResourceDef, useResourceRecordId } from '@/hooks/useResourceDef'
import { resourceNesteds } from '@/resources/definitions'
import { fieldValue, formatFieldValue, rowLabel } from '@/lib/resourceForm'
import { resourceEditTo, resourceListTo } from '@/lib/resourcePaths'

export function ResourceShowPage() {
  const def = useResourceDef()
  const recordId = useResourceRecordId()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const api = resourceApi<Record<string, unknown>>(def.path)
  const allowDelete = def.allowDelete !== false

  const [deleteOpen, setDeleteOpen] = useState(false)

  const { data: row, isLoading, isError } = useQuery({
    queryKey: [def.key, 'detail', recordId],
    queryFn: () => api.get(recordId!),
    enabled: recordId != null,
  })

  const deleteMutation = useMutation({
    mutationFn: () => api.remove(recordId!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [def.key] })
      navigate(resourceListTo(def))
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
    <div className="space-y-6">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <Button variant="ghost" size="sm" asChild className="mb-2 -ml-2">
            <Link {...resourceListTo(def)}>← Voltar à lista</Link>
          </Button>
          <h1 className="text-2xl font-semibold tracking-tight">{def.title}</h1>
          <p className="text-sm text-muted-foreground">
            ID {String(row.id)} · {rowLabel(row, def.fields)}
          </p>
        </div>
        <div className="flex flex-wrap items-start justify-end gap-2">
          {def.key === 'documentos' && (
            <DocumentoXmlDownloadButton documentoId={recordId} row={row} />
          )}
          <Button variant="outline" asChild>
            <Link {...resourceEditTo(def, recordId)}>Editar</Link>
          </Button>
          {allowDelete && (
            <Button variant="destructive" onClick={() => setDeleteOpen(true)}>
              Excluir
            </Button>
          )}
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Detalhes</CardTitle>
        </CardHeader>
        <CardContent>
          <dl className="grid gap-3 sm:grid-cols-2">
            {def.fields
              .filter((field) => !field.formOnly)
              .map((field) => (
                <div key={field.key}>
                  <dt className="text-sm font-medium text-muted-foreground">{field.label}</dt>
                  <dd className="text-sm">
                    {formatFieldValue(fieldValue(row, field.key), field, row)}
                  </dd>
                </div>
              ))}
          </dl>
        </CardContent>
      </Card>

      {def.key === 'documentos' && <DocumentoPartesShowCard row={row} />}

      {resourceNesteds(def).map((nested) => (
        <NestedResourcePanel
          key={nested.path(recordId)}
          parentId={recordId}
          nested={nested}
        />
      ))}

      <DeleteConfirmDialog
        open={deleteOpen}
        onOpenChange={setDeleteOpen}
        title={`Excluir ${def.title.toLowerCase()}?`}
        description={`Esta ação não pode ser desfeita. Registro: ${rowLabel(row, def.fields)} (ID ${row.id}).`}
        pending={deleteMutation.isPending}
        onConfirm={() => deleteMutation.mutate()}
      />
    </div>
  )
}
