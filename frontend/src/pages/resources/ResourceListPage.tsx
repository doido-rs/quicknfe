import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link } from '@tanstack/react-router'
import { useCallback, useState } from 'react'
import { resourceApi } from '@/api/client'
import { DeleteConfirmDialog } from '@/components/resources/DeleteConfirmDialog'
import { ResourceDataTable } from '@/components/resources/ResourceDataTable'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useResourceDef } from '@/hooks/useResourceDef'
import { rowLabel } from '@/lib/resourceForm'
import { resourceNewTo } from '@/lib/resourcePaths'

export function ResourceListPage() {
  const def = useResourceDef()
  const queryClient = useQueryClient()
  const api = resourceApi<Record<string, unknown>>(def.path)
  const allowCreate = def.allowCreate !== false
  const allowDelete = def.allowDelete !== false

  const [deleteTarget, setDeleteTarget] = useState<Record<string, unknown> | null>(null)

  const { data: rows = [], isLoading } = useQuery({
    queryKey: [def.key, 'list'],
    queryFn: () => api.list(),
  })

  const deleteMutation = useMutation({
    mutationFn: (id: number) => api.remove(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [def.key] })
      setDeleteTarget(null)
    },
  })

  const onDelete = useCallback((row: Record<string, unknown>) => {
    setDeleteTarget(row)
  }, [])

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-40 w-full" />
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h1 className="text-2xl font-semibold tracking-tight">{def.title}</h1>
        {allowCreate && (
          <Button asChild>
            <Link {...resourceNewTo(def)}>Novo</Link>
          </Button>
        )}
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Lista</CardTitle>
        </CardHeader>
        <CardContent>
          <ResourceDataTable
            def={def}
            rows={rows}
            allowDelete={allowDelete}
            onDelete={onDelete}
          />
        </CardContent>
      </Card>

      <DeleteConfirmDialog
        open={deleteTarget != null}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        title={`Excluir ${def.title.toLowerCase()}?`}
        description={
          deleteTarget
            ? `Esta ação não pode ser desfeita. Registro: ${rowLabel(deleteTarget, def.fields)} (ID ${deleteTarget.id}).`
            : ''
        }
        pending={deleteMutation.isPending}
        onConfirm={() => {
          if (deleteTarget?.id) {
            deleteMutation.mutate(Number(deleteTarget.id))
          }
        }}
      />
    </div>
  )
}
