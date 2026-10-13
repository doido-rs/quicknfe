import { useInfiniteQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Link } from '@tanstack/react-router'
import { useCallback, useEffect, useState } from 'react'
import { resourceApi } from '@/api/client'
import { PageTitle } from '@/components/layout/PageTitle'
import { DeleteConfirmDialog } from '@/components/resources/DeleteConfirmDialog'
import { DocumentoListTable } from '@/components/resources/DocumentoListTable'
import { ResourceDataTable } from '@/components/resources/ResourceDataTable'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useResourceDef } from '@/hooks/useResourceDef'
import { rowLabel } from '@/lib/resourceForm'
import { appSettingsTo, resourceNewTo } from '@/lib/resourcePaths'
import { toastFromError, toastSuccess } from '@/lib/toast'

export function ResourceListPage() {
  const def = useResourceDef()
  const queryClient = useQueryClient()
  const api = resourceApi<Record<string, unknown>>(def.path)
  const allowCreate = def.allowCreate !== false
  const allowDelete = def.allowDelete !== false

  const [deleteTarget, setDeleteTarget] = useState<Record<string, unknown> | null>(null)

  const {
    data,
    isLoading,
    isError,
    error,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery({
    queryKey: [def.key, 'list'],
    queryFn: ({ pageParam }) =>
      api.listPage({ pageToken: pageParam as string | undefined, pageSize: 20 }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (last) => last.next_page_token ?? undefined,
  })

  const rows = data?.pages.flatMap((p) => p.items) ?? []

  const deleteMutation = useMutation({
    mutationFn: (id: number) => api.remove(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [def.key] })
      setDeleteTarget(null)
      toastSuccess('Registro excluído')
    },
    onError: (err) => toastFromError(err, 'Não foi possível excluir'),
  })

  useEffect(() => {
    if (isError) {
      toastFromError(error, 'Não foi possível carregar a lista')
    }
  }, [isError, error])

  const onDelete = useCallback((row: Record<string, unknown>) => {
    setDeleteTarget(row)
  }, [])

  if (isLoading) {
    return (
      <div className="space-y-4">
        <PageTitle>{def.title}</PageTitle>
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-40 w-full" />
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <PageTitle>{def.title}</PageTitle>
      {def.menuPlacement === 'settings' && (
        <Button variant="ghost" size="sm" asChild className="-ml-2">
          <Link {...appSettingsTo}>← Configurações</Link>
        </Button>
      )}
      {allowCreate && (
        <div className="flex flex-wrap items-center justify-end gap-2">
          <Button asChild>
            <Link {...resourceNewTo(def)}>Novo</Link>
          </Button>
        </div>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Lista</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {def.key === 'documentos' ? (
            <DocumentoListTable
              def={def}
              rows={rows}
              allowDelete={allowDelete}
              onDelete={onDelete}
            />
          ) : (
            <ResourceDataTable
              def={def}
              rows={rows}
              allowDelete={allowDelete}
              onDelete={onDelete}
            />
          )}
          {hasNextPage && (
            <div className="flex justify-center">
              <Button
                type="button"
                variant="outline"
                disabled={isFetchingNextPage}
                onClick={() => fetchNextPage()}
              >
                {isFetchingNextPage ? 'Carregando…' : 'Carregar mais'}
              </Button>
            </div>
          )}
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
