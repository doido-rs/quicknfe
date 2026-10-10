import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useState } from 'react'
import { Link, Navigate, useNavigate, useParams } from 'react-router-dom'
import { resourceApi } from '@/api/client'
import { DeleteConfirmDialog } from '@/components/resources/DeleteConfirmDialog'
import { NestedResourcePanel } from '@/components/resources/NestedResourcePanel'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useResourceDef } from '@/hooks/useResourceDef'
import { formatFieldValue, rowLabel } from '@/lib/resourceForm'
import { resourceEditPath, resourceListPath } from '@/lib/resourcePaths'

export function ResourceShowPage() {
  const def = useResourceDef()
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const recordId = Number(id)
  const api = resourceApi<Record<string, unknown>>(def.path)
  const allowDelete = def.allowDelete !== false

  const [deleteOpen, setDeleteOpen] = useState(false)

  const { data: row, isLoading, isError } = useQuery({
    queryKey: [def.key, 'detail', recordId],
    queryFn: () => api.get(recordId),
    enabled: Number.isFinite(recordId),
  })

  const deleteMutation = useMutation({
    mutationFn: () => api.remove(recordId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [def.key] })
      navigate(resourceListPath(def))
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

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <Button variant="ghost" size="sm" asChild className="mb-2 -ml-2">
            <Link to={resourceListPath(def)}>← Voltar à lista</Link>
          </Button>
          <h1 className="text-2xl font-semibold tracking-tight">{def.title}</h1>
          <p className="text-sm text-muted-foreground">
            ID {String(row.id)} · {rowLabel(row, def.fields)}
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" asChild>
            <Link to={resourceEditPath(def, recordId)}>Editar</Link>
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
            {def.fields.map((field) => (
              <div key={field.key}>
                <dt className="text-sm font-medium text-muted-foreground">{field.label}</dt>
                <dd className="text-sm">{formatFieldValue(row[field.key])}</dd>
              </div>
            ))}
          </dl>
        </CardContent>
      </Card>

      {def.nested && (
        <NestedResourcePanel parentId={recordId} nested={def.nested} />
      )}

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
