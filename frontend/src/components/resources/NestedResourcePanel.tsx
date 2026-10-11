import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useState } from 'react'
import { resourceApi } from '@/api/client'
import { ResourceFieldInput } from '@/components/resources/ResourceFieldInput'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  buildCreateBodyWithFiles,
  emptyForm,
  rowLabel,
  useResourceFieldForm,
} from '@/lib/resourceForm'
import type { ResourceDef } from '@/resources/definitions'

export function NestedResourcePanel({
  parentId,
  nested,
}: {
  parentId: number
  nested: NonNullable<ResourceDef['nested']>
}) {
  const path = nested.path(parentId)
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

  const createMutation = useMutation({
    mutationFn: async (values: Record<string, string>) => {
      for (const f of nested.fields) {
        if (f.type === 'file' && f.required && !files[f.key]) {
          throw new Error(`Selecione o arquivo: ${f.label}`)
        }
      }
      const body = await buildCreateBodyWithFiles(nested.fields, values, files)
      return api.create(body)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [path] })
      reset(emptyForm(nested.fields))
      setFiles({})
      setSubmitError(null)
    },
    onError: (err) => {
      setSubmitError(err instanceof Error ? err.message : 'Falha ao enviar')
    },
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{nested.label}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
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
        <ul className="list-disc space-y-1 pl-5 text-sm">
          {rows.map((r) => (
            <li key={String(r.id)}>{rowLabel(r, nested.fields)}</li>
          ))}
        </ul>
      </CardContent>
    </Card>
  )
}
