import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import { resourceApi } from '@/api/client'
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
import { toastFromError, toastSuccess } from '@/lib/toast'
import type { NestedDef } from '@/resources/definitions'

export function NestedResourcePanel({
  parentId,
  nested,
}: {
  parentId: number
  nested: NestedDef
}) {
  const path = nested.path(parentId)
  const allowCreate = nested.allowCreate !== false
  const api = resourceApi<Record<string, unknown>>(path)
  const queryClient = useQueryClient()
  const form = useResourceFieldForm(nested.fields)
  const { handleSubmit, reset } = form
  const [files, setFiles] = useState<Record<string, File | undefined>>({})
  const [submitError, setSubmitError] = useState<string | null>(null)

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
            {nested.fields.map((f) => (
              <ResourceFieldInput
                key={f.key}
                field={f}
                form={form}
                idPrefix={`nested-${f.key}`}
                attachedFile={files[f.key]}
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
          {rows.map((r) => (
            <li key={String(r.id)}>{rowLabel(r, nested.fields)}</li>
          ))}
        </ul>
      </CardContent>
    </Card>
  )
}
