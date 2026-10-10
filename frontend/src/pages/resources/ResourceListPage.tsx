import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useState } from 'react'
import { Link } from 'react-router-dom'
import { resourceApi } from '@/api/client'
import { DeleteConfirmDialog } from '@/components/resources/DeleteConfirmDialog'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { useResourceDef } from '@/hooks/useResourceDef'
import { rowLabel } from '@/lib/resourceForm'
import {
  resourceEditPath,
  resourceNewPath,
  resourceShowPath,
} from '@/lib/resourcePaths'

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
            <Link to={resourceNewPath(def)}>Novo</Link>
          </Button>
        )}
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Lista</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>ID</TableHead>
                <TableHead>Resumo</TableHead>
                <TableHead className="text-right">Ações</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {rows.length === 0 && (
                <TableRow>
                  <TableCell colSpan={3} className="text-muted-foreground">
                    Nenhum registro.
                  </TableCell>
                </TableRow>
              )}
              {rows.map((row) => (
                <TableRow key={String(row.id)}>
                  <TableCell>{String(row.id)}</TableCell>
                  <TableCell>{rowLabel(row, def.fields)}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex flex-wrap justify-end gap-2">
                      <Button type="button" variant="outline" size="sm" asChild>
                        <Link to={resourceShowPath(def, Number(row.id))}>Ver</Link>
                      </Button>
                      <Button type="button" variant="outline" size="sm" asChild>
                        <Link to={resourceEditPath(def, Number(row.id))}>Editar</Link>
                      </Button>
                      {allowDelete && (
                        <Button
                          type="button"
                          variant="destructive"
                          size="sm"
                          onClick={() => setDeleteTarget(row)}
                        >
                          Excluir
                        </Button>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
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
