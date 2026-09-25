import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect, useState, type FormEvent } from 'react'
import { resourceApi, singletonResourceApi } from '@/api/client'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { buildBody, emptyForm, rowLabel } from '@/lib/resourceForm'
import type { ResourceDef } from '@/resources/definitions'

export function ResourceCrud({ def }: { def: ResourceDef }) {
  const isSingleton = def.key === 'empresa'
  const api = resourceApi<Record<string, unknown>>(def.path)
  const singleton = singletonResourceApi<Record<string, unknown>>(def.path)
  const queryClient = useQueryClient()
  const [editing, setEditing] = useState<Record<string, unknown> | null>(null)
  const [form, setForm] = useState(() => emptyForm(def.fields))
  const [nestedParent, setNestedParent] = useState<number | null>(null)
  const allowCreate = def.allowCreate !== false

  const { data: rows = [], isLoading } = useQuery({
    queryKey: [def.key, 'list'],
    queryFn: () => api.list(),
    enabled: !isSingleton,
  })

  const { data: singletonRow, isLoading: singletonLoading } = useQuery({
    queryKey: [def.key, 'singleton'],
    queryFn: () => singleton.get(),
    enabled: isSingleton,
  })

  useEffect(() => {
    if (isSingleton && singletonRow) {
      setForm(
        Object.fromEntries(
          def.fields.map((f) => [
            f.key,
            singletonRow[f.key] != null ? String(singletonRow[f.key]) : '',
          ]),
        ),
      )
    }
  }, [isSingleton, singletonRow, def.fields])

  const saveMutation = useMutation({
    mutationFn: async () => {
      const body = buildBody(def.fields, form)
      if (isSingleton) {
        return singleton.update(body)
      }
      if (editing?.id) {
        return api.update(Number(editing.id), body)
      }
      return api.create(body)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [def.key] })
      if (!isSingleton) {
        setEditing(null)
        setForm(emptyForm(def.fields))
      }
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: number) => api.remove(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: [def.key] }),
  })

  function startEdit(row: Record<string, unknown>) {
    setEditing(row)
    setForm(
      Object.fromEntries(
        def.fields.map((f) => [f.key, row[f.key] != null ? String(row[f.key]) : '']),
      ),
    )
  }

  function onSubmit(e: FormEvent) {
    e.preventDefault()
    if (!allowCreate && !editing && !isSingleton) {
      return
    }
    saveMutation.mutate()
  }

  if ((isLoading || singletonLoading) && !singletonRow) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-40 w-full" />
      </div>
    )
  }

  const submitLabel = isSingleton
    ? 'Salvar empresa'
    : editing
      ? 'Salvar'
      : allowCreate
        ? 'Criar'
        : 'Selecione um registro para editar'

  const showSubmit = isSingleton || editing != null || allowCreate

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>{def.title}</CardTitle>
          {!showSubmit && !isSingleton && (
            <CardDescription>Selecione um registro para editar</CardDescription>
          )}
        </CardHeader>
        <CardContent>
          <form onSubmit={onSubmit} className="grid gap-4 md:grid-cols-2">
            {def.fields.map((field) => (
              <div key={field.key} className="grid gap-2">
                <Label htmlFor={`field-${field.key}`}>{field.label}</Label>
                <Input
                  id={`field-${field.key}`}
                  required={field.required && (isSingleton || editing != null || allowCreate)}
                  type={
                    field.type === 'number'
                      ? 'number'
                      : field.type === 'date'
                        ? 'datetime-local'
                        : 'text'
                  }
                  value={form[field.key] ?? ''}
                  onChange={(ev) =>
                    setForm((prev) => ({ ...prev, [field.key]: ev.target.value }))
                  }
                />
              </div>
            ))}
            {showSubmit && (
              <div className="md:col-span-2">
                <Button type="submit" disabled={saveMutation.isPending}>
                  {submitLabel}
                </Button>
              </div>
            )}
          </form>
        </CardContent>
      </Card>

      {!isSingleton && (
        <Card>
          <CardContent className="pt-6">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>ID</TableHead>
                  <TableHead>Resumo</TableHead>
                  <TableHead className="text-right">Ações</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {rows.map((row) => (
                  <TableRow key={String(row.id)}>
                    <TableCell>{String(row.id)}</TableCell>
                    <TableCell>{rowLabel(row, def.fields)}</TableCell>
                    <TableCell className="flex flex-wrap justify-end gap-2">
                      <Button type="button" variant="outline" size="sm" onClick={() => startEdit(row)}>
                        Editar
                      </Button>
                      {allowCreate && (
                        <Button
                          type="button"
                          variant="destructive"
                          size="sm"
                          onClick={() => deleteMutation.mutate(Number(row.id))}
                        >
                          Excluir
                        </Button>
                      )}
                      {def.nested && (
                        <Button
                          type="button"
                          variant="secondary"
                          size="sm"
                          onClick={() => setNestedParent(Number(row.id))}
                        >
                          {def.nested.label}
                        </Button>
                      )}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      )}

      {nestedParent != null && def.nested && (
        <NestedCrud
          parentId={nestedParent}
          nested={def.nested}
          onClose={() => setNestedParent(null)}
        />
      )}
    </div>
  )
}

function NestedCrud({
  parentId,
  nested,
  onClose,
}: {
  parentId: number
  nested: NonNullable<ResourceDef['nested']>
  onClose: () => void
}) {
  const path = nested.path(parentId)
  const api = resourceApi<Record<string, unknown>>(path)
  const queryClient = useQueryClient()
  const [form, setForm] = useState(() => emptyForm(nested.fields))

  const { data: rows = [] } = useQuery({
    queryKey: [path],
    queryFn: () => api.list(),
  })

  const createMutation = useMutation({
    mutationFn: () => api.create(buildBody(nested.fields, form)),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [path] })
      setForm(emptyForm(nested.fields))
    },
  })

  return (
    <Card>
      <CardHeader className="flex flex-row items-start justify-between gap-4">
        <CardTitle className="text-base">
          {nested.label} (doc #{parentId})
        </CardTitle>
        <Button type="button" variant="outline" size="sm" onClick={onClose}>
          Fechar
        </Button>
      </CardHeader>
      <CardContent className="space-y-4">
        <form
          className="grid gap-4 md:grid-cols-2"
          onSubmit={(e) => {
            e.preventDefault()
            createMutation.mutate()
          }}
        >
          {nested.fields.map((f) => (
            <div key={f.key} className="grid gap-2">
              <Label htmlFor={`nested-${f.key}`}>{f.label}</Label>
              <Input
                id={`nested-${f.key}`}
                type={f.type === 'number' ? 'number' : 'text'}
                value={form[f.key] ?? ''}
                onChange={(ev) => setForm((p) => ({ ...p, [f.key]: ev.target.value }))}
              />
            </div>
          ))}
          <div className="md:col-span-2">
            <Button type="submit">Adicionar item</Button>
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
