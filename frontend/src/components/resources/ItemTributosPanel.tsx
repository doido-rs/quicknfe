import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ChevronDown, ChevronRight } from 'lucide-react'
import { useState } from 'react'
import { resourceApi } from '@/api/client'
import { ResourceFieldInput } from '@/components/resources/ResourceFieldInput'
import { Button } from '@/components/ui/button'
import {
  buildNestedCreateBody,
  emptyForm,
  rowLabel,
  useResourceFieldForm,
} from '@/lib/resourceForm'
import { toastFromError, toastSuccess } from '@/lib/toast'
import type { ItemTributosDef } from '@/resources/definitions'

export function ItemTributosPanel({
  itemId,
  itemLabel,
  def,
}: {
  itemId: number
  itemLabel: string
  def: ItemTributosDef
}) {
  const [open, setOpen] = useState(false)
  const path = def.path(itemId)
  const api = resourceApi<Record<string, unknown>>(path)
  const queryClient = useQueryClient()
  const form = useResourceFieldForm(def.fields)
  const { handleSubmit, reset } = form

  const { data: page } = useQuery({
    queryKey: [path, 'list'],
    queryFn: () => api.listPage({ pageSize: 20 }),
    enabled: open,
  })
  const rows = page?.items ?? []

  const createMutation = useMutation({
    mutationFn: (values: Record<string, string>) => {
      const body = buildNestedCreateBody({ fields: def.fields }, values)
      return api.create(body)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [path] })
      reset(emptyForm(def.fields))
      toastSuccess('Tributo adicionado')
    },
    onError: (err) => toastFromError(err, 'Não foi possível salvar tributo'),
  })

  return (
    <li className="list-none pl-0">
      <button
        type="button"
        className="flex w-full items-center gap-1 text-left text-sm font-medium text-muted-foreground hover:text-foreground"
        onClick={() => setOpen((v) => !v)}
      >
        {open ? <ChevronDown className="size-4" /> : <ChevronRight className="size-4" />}
        {def.label ?? 'Tributos'} — {itemLabel}
      </button>
      {open && (
        <div className="mt-2 space-y-3 rounded-md border border-border/60 bg-muted/20 p-3">
          <form
            className="grid gap-3 md:grid-cols-2"
            onSubmit={handleSubmit((values) => createMutation.mutate(values))}
          >
            {def.fields.map((f) => (
              <ResourceFieldInput
                key={f.key}
                field={f}
                form={form}
                idPrefix={`trib-${itemId}-${f.key}`}
              />
            ))}
            <div className="md:col-span-2">
              <Button type="submit" size="sm" disabled={createMutation.isPending}>
                Adicionar tributo
              </Button>
            </div>
          </form>
          <ul className="list-disc space-y-1 pl-5 text-sm">
            {rows.length === 0 && (
              <li className="list-none pl-0 text-muted-foreground">Nenhum tributo.</li>
            )}
            {rows.map((r) => (
              <li key={String(r.id)}>{rowLabel(r, def.fields)}</li>
            ))}
          </ul>
        </div>
      )}
    </li>
  )
}
