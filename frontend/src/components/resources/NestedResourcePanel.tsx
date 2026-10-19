import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useState } from 'react'
import { resourceApi } from '@/api/client'
import { DeleteConfirmDialog } from '@/components/resources/DeleteConfirmDialog'
import { ResourceFieldInput } from '@/components/resources/ResourceFieldInput'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  buildCreateBodyWithFiles,
  buildNestedCreateBody,
  emptyForm,
  rowLabel,
  useResourceFieldForm,
} from '@/lib/resourceForm'
import { formatDateForDisplay, normalizeApiDateToForm } from '@/lib/dateFormat'
import { toastFromError, toastSuccess } from '@/lib/toast'
import { useUnsavedFormGuard } from '@/hooks/useUnsavedFormGuard'
import { useDateFormat } from '@/theme/DateFormatContext'
import { ItemTributosPanel } from '@/components/resources/ItemTributosPanel'
import type { NestedDef } from '@/resources/definitions'

export function NestedResourcePanel({
  parentId,
  nested,
  relationScopeValues,
}: {
  parentId: number
  nested: NestedDef
  relationScopeValues?: Record<string, string>
}) {
  const path = nested.path(parentId)
  const allowCreate = nested.allowCreate !== false
  const allowDelete = nested.allowDelete === true
  const api = resourceApi<Record<string, unknown>>(path)
  const queryClient = useQueryClient()
  const dateFormat = useDateFormat()
  const form = useResourceFieldForm(nested.fields)
  const {
    handleSubmit,
    reset,
    formState: { isDirty },
  } = form
  useUnsavedFormGuard(allowCreate && isDirty)
  const [files, setFiles] = useState<Record<string, File | undefined>>({})
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<Record<string, unknown> | null>(null)

  const { data: page } = useQuery({
    queryKey: [path, 'list'],
    queryFn: () => api.listPage({ pageSize: 50 }),
  })
  const rows = page?.items ?? []

  useEffect(() => {
    if (!nested.fields.some((f) => f.key === 'numero_item')) return
    const next = rows.length + 1
    const current = form.getValues('numero_item')
    if (current === '' || current === String(Math.max(0, next - 1))) {
      form.setValue('numero_item', String(next))
    }
  }, [rows.length, nested.fields, form])

  const createMutation = useMutation({
    mutationFn: async (values: Record<string, string>) => {
      for (const f of nested.fields) {
        if (f.type === 'file' && f.required && !files[f.key]) {
          throw new Error(`Selecione o arquivo: ${f.label}`)
        }
      }
      const hasFiles = nested.fields.some((f) => f.type === 'file')
      const body = hasFiles
        ? await buildCreateBodyWithFiles(nested.fields, values, files)
        : buildNestedCreateBody(nested, values)
      return api.create(body)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [path] })
      reset(emptyForm(nested.fields))
      setFiles({})
      setSubmitError(null)
      toastSuccess('Registro criado com sucesso')
    },
    onError: (err) => {
      const msg = err instanceof Error ? err.message : 'Falha ao enviar'
      setSubmitError(msg)
      toastFromError(err, 'Não foi possível criar o registro')
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: number) => api.remove(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [path] })
      setDeleteTarget(null)
      toastSuccess('Registro excluído')
    },
    onError: (err) => toastFromError(err, 'Não foi possível excluir'),
  })

  const onDeleteClick = useCallback((row: Record<string, unknown>) => {
    setDeleteTarget(row)
  }, [])

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{nested.label}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {allowCreate && (
          <form
            className="grid gap-4 md:grid-cols-2"
            onSubmit={handleSubmit((values) => createMutation.mutate(values))}
          >
            {nested.createHint && (
              <p className="md:col-span-2 text-sm text-muted-foreground">{nested.createHint}</p>
            )}
            {nested.fields.map((f) => (
              <ResourceFieldInput
                key={f.key}
                field={f}
                form={form}
                idPrefix={`nested-${f.key}`}
                attachedFile={files[f.key]}
                relationScopeValues={relationScopeValues}
                onFileChange={(key, file) =>
                  setFiles((prev) => ({ ...prev, [key]: file ?? undefined }))
                }
              />
            ))}
            <div className="md:col-span-2">
              {submitError && <p className="mb-2 text-sm text-destructive">{submitError}</p>}
              <Button type="submit" disabled={createMutation.isPending}>
                {nested.createButtonLabel ?? 'Adicionar item'}
              </Button>
            </div>
          </form>
        )}
        <ul className="list-disc space-y-1 pl-5 text-sm">
          {rows.length === 0 && !allowCreate && (
            <li className="list-none pl-0 text-muted-foreground">Nenhum registro.</li>
          )}
          {rows.map((r) => {
            const id = Number(r.id)
            const label = rowLabel(r, nested.fields)
            return (
              <li key={String(r.id)} className="list-none space-y-1 pl-0">
                <div className="flex flex-wrap items-center gap-2">
                  <span>
                    {label}
                    {nested.listColumns?.map((col) => {
                      const raw = r[col.key]
                      if (raw == null || String(raw).trim() === '') return null
                      const display =
                        col.type === 'date'
                          ? formatDateForDisplay(normalizeApiDateToForm(String(raw)), dateFormat)
                          : String(raw)
                      return (
                        <span key={col.key} className="text-muted-foreground">
                          {' '}
                          — {col.label}: {display}
                        </span>
                      )
                    })}
                  </span>
                  {allowDelete && Number.isFinite(id) && (
                    <Button
                      type="button"
                      variant="destructive"
                      size="sm"
                      onClick={() => onDeleteClick(r)}
                    >
                      Excluir
                    </Button>
                  )}
                </div>
                {nested.itemTributos && Number.isFinite(id) && (
                  <ItemTributosPanel
                    itemId={id}
                    itemLabel={label}
                    def={nested.itemTributos}
                  />
                )}
              </li>
            )
          })}
        </ul>
        <DeleteConfirmDialog
          open={deleteTarget != null}
          onOpenChange={(open) => {
            if (!open) setDeleteTarget(null)
          }}
          description={
            deleteTarget
              ? `Esta ação não pode ser desfeita. Registro: ${rowLabel(deleteTarget, nested.fields)} (ID ${deleteTarget.id}).`
              : undefined
          }
          pending={deleteMutation.isPending}
          onConfirm={() => {
            if (deleteTarget?.id) {
              deleteMutation.mutate(Number(deleteTarget.id))
            }
          }}
        />
      </CardContent>
    </Card>
  )
}
