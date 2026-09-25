import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect, useState, type FormEvent } from 'react'
import { resourceApi, singletonResourceApi } from '../api/client'
import { buildBody, emptyForm, rowLabel } from '../lib/resourceForm'
import type { ResourceDef } from '../resources/definitions'

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
    return <p>Carregando…</p>
  }

  const submitLabel = isSingleton
    ? 'Salvar empresa'
    : editing
      ? 'Salvar'
      : allowCreate
        ? 'Criar'
        : 'Selecione um registro para editar'

  return (
    <section className="resource-crud">
      <h2>{def.title}</h2>
      <form onSubmit={onSubmit} className="form-grid">
        {def.fields.map((field) => (
          <label key={field.key}>
            {field.label}
            <input
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
          </label>
        ))}
        {(isSingleton || editing || allowCreate) && (
          <button type="submit" disabled={saveMutation.isPending}>
            {submitLabel}
          </button>
        )}
      </form>

      {!isSingleton && (
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Resumo</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={String(row.id)}>
                <td>{String(row.id)}</td>
                <td>{rowLabel(row, def.fields)}</td>
                <td>
                  <button type="button" onClick={() => startEdit(row)}>
                    Editar
                  </button>
                  {allowCreate && (
                    <button
                      type="button"
                      onClick={() => deleteMutation.mutate(Number(row.id))}
                    >
                      Excluir
                    </button>
                  )}
                  {def.nested && (
                    <button
                      type="button"
                      onClick={() => setNestedParent(Number(row.id))}
                    >
                      {def.nested.label}
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {nestedParent != null && def.nested && (
        <NestedCrud
          parentId={nestedParent}
          nested={def.nested}
          onClose={() => setNestedParent(null)}
        />
      )}
    </section>
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
    <div className="nested-panel">
      <h3>
        {nested.label} (doc #{parentId})
        <button type="button" onClick={onClose}>
          Fechar
        </button>
      </h3>
      <form
        onSubmit={(e) => {
          e.preventDefault()
          createMutation.mutate()
        }}
      >
        {nested.fields.map((f) => (
          <label key={f.key}>
            {f.label}
            <input
              value={form[f.key] ?? ''}
              onChange={(ev) => setForm((p) => ({ ...p, [f.key]: ev.target.value }))}
            />
          </label>
        ))}
        <button type="submit">Adicionar item</button>
      </form>
      <ul>
        {rows.map((r) => (
          <li key={String(r.id)}>{rowLabel(r, nested.fields)}</li>
        ))}
      </ul>
    </div>
  )
}
