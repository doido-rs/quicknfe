import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { FormEvent, useState } from 'react'
import { resourceApi } from '../api/client'
import type { FieldDef, ResourceDef } from '../resources/definitions'

function emptyForm(fields: FieldDef[]): Record<string, string> {
  return Object.fromEntries(fields.map((f) => [f.key, '']))
}

function rowLabel(row: Record<string, unknown>, fields: FieldDef[]): string {
  const first = fields.find((f) => row[f.key] != null)
  return first ? String(row[first.key]) : String(row.id ?? '')
}

export function ResourceCrud({ def }: { def: ResourceDef }) {
  const api = resourceApi<Record<string, unknown>>(def.path)
  const queryClient = useQueryClient()
  const [editing, setEditing] = useState<Record<string, unknown> | null>(null)
  const [form, setForm] = useState(() => emptyForm(def.fields))
  const [nestedParent, setNestedParent] = useState<number | null>(null)

  const { data: rows = [], isLoading } = useQuery({
    queryKey: [def.key],
    queryFn: () => (def.key === 'empresa' ? api.get(0).then((r) => [r]) : api.list()),
    enabled: def.key !== 'empresa',
  })

  const { data: empresaSingle } = useQuery({
    queryKey: [def.key],
    queryFn: () => api.get(0),
    enabled: def.key === 'empresa',
  })

  const list = def.key === 'empresa' && empresaSingle ? [empresaSingle] : rows

  const saveMutation = useMutation({
    mutationFn: async () => {
      const body = Object.fromEntries(
        def.fields.map((f) => {
          const raw = form[f.key]
          if (f.type === 'number' && raw !== '') {
            return [f.key, Number(raw)]
          }
          if (raw === '') {
            return [f.key, null]
          }
          return [f.key, raw]
        }),
      )
      if (def.key === 'empresa') {
        return api.update(0, body)
      }
      if (editing?.id) {
        return api.update(Number(editing.id), body)
      }
      return api.create(body)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [def.key] })
      setEditing(null)
      setForm(emptyForm(def.fields))
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
    saveMutation.mutate()
  }

  if (isLoading && def.key !== 'empresa') {
    return <p>Carregando…</p>
  }

  return (
    <section>
      <h2>{def.title}</h2>
      <form onSubmit={onSubmit} className="form-grid">
        {def.fields.map((field) => (
          <label key={field.key}>
            {field.label}
            <input
              required={field.required}
              type={field.type === 'number' ? 'number' : 'text'}
              value={form[field.key] ?? ''}
              onChange={(ev) =>
                setForm((prev) => ({ ...prev, [field.key]: ev.target.value }))
              }
            />
          </label>
        ))}
        <button type="submit" disabled={saveMutation.isPending}>
          {editing || def.key === 'empresa' ? 'Salvar' : 'Criar'}
        </button>
      </form>

      {def.key !== 'empresa' && (
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Resumo</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            {list.map((row) => (
              <tr key={String(row.id)}>
                <td>{String(row.id)}</td>
                <td>{rowLabel(row, def.fields)}</td>
                <td>
                  <button type="button" onClick={() => startEdit(row)}>
                    Editar
                  </button>
                  <button
                    type="button"
                    onClick={() => deleteMutation.mutate(Number(row.id))}
                  >
                    Excluir
                  </button>
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
    mutationFn: () => {
      const body = Object.fromEntries(
        nested.fields.map((f) => [
          f.key,
          f.type === 'number' && form[f.key] !== '' ? Number(form[f.key]) : form[f.key],
        ]),
      )
      return api.create(body)
    },
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
