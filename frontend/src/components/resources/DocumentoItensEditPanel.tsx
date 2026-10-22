import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Fragment, useCallback, useEffect, useMemo, useState } from 'react'
import { resourceApi } from '@/api/client'
import { DeleteConfirmDialog } from '@/components/resources/DeleteConfirmDialog'
import { ItemTributosPanel } from '@/components/resources/ItemTributosPanel'
import { ResourceFieldInput } from '@/components/resources/ResourceFieldInput'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  buildNestedCreateBody,
  emptyForm,
  rowToForm,
  useResourceFieldForm,
} from '@/lib/resourceForm'
import { toastFromError, toastSuccess } from '@/lib/toast'
import { useUnsavedFormGuard } from '@/hooks/useUnsavedFormGuard'
import type { FieldDef, NestedDef } from '@/resources/definitions'
import { formatMoneyForDisplay, normalizeMoneyToForm, roundMoney } from '@/lib/moneyFormat'
import { cn } from '@/lib/utils'

const COMPACT_ITEM_LAYOUT: { keys: string[]; className?: string }[] = [
  { keys: ['numero_item', 'quantidade', 'valor_unitario', 'valor_desconto', 'valor_total_bruto'] },
  { keys: ['conta_fiscal_id', 'cna_id', 'codigo_tributacao_municipal', 'codigo_ncm'] },
  { keys: ['descricao_servico'], className: 'col-span-full' },
]

function compactFieldLabel(field: FieldDef): FieldDef {
  const short: Record<string, string> = {
    numero_item: 'Nº',
    descricao_servico: 'Descrição do serviço',
    codigo_tributacao_municipal: 'Cód. municipal (cTribMun)',
    valor_unitario: 'V. unitário',
    valor_total_bruto: 'Total bruto',
    valor_desconto: 'Desconto',
    conta_fiscal_id: 'Conta fiscal',
    cna_id: 'CNA nacional',
    codigo_ncm: 'NCM',
    quantidade: 'Qtd.',
  }
  return { ...field, label: short[field.key] ?? field.label }
}

function truncate(text: string, max: number): string {
  const t = text.trim()
  if (t.length <= max) return t
  return `${t.slice(0, max - 1)}…`
}

function ItemFieldsGrid({
  fieldsByKey,
  form,
  idPrefix,
  relationScopeValues,
}: {
  fieldsByKey: Map<string, FieldDef>
  form: ReturnType<typeof useResourceFieldForm>
  idPrefix: string
  relationScopeValues?: Record<string, string>
}) {
  return (
    <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
      {COMPACT_ITEM_LAYOUT.flatMap((group) =>
        group.keys.map((key) => {
          const field = fieldsByKey.get(key)
          if (!field) return null
          const isTextarea = field.key === 'descricao_servico'
          return (
            <div
              key={field.key}
              className={cn(isTextarea ? 'sm:col-span-2 lg:col-span-4' : '', group.className)}
            >
              <ResourceFieldInput
                field={field}
                form={form}
                idPrefix={`${idPrefix}-${field.key}`}
                relationScopeValues={relationScopeValues}
                dense
              />
            </div>
          )
        }),
      )}
    </div>
  )
}

export function DocumentoItensEditPanel({
  parentId,
  nested,
  relationScopeValues,
}: {
  parentId: number
  nested: NestedDef
  relationScopeValues?: Record<string, string>
}) {
  const path = nested.path(parentId)
  const api = resourceApi<Record<string, unknown>>(path)
  const queryClient = useQueryClient()
  const form = useResourceFieldForm(nested.fields)
  const {
    handleSubmit,
    reset,
    watch,
    setValue,
    formState: { isDirty },
  } = form
  useUnsavedFormGuard(isDirty)

  const [showAddForm, setShowAddForm] = useState(false)
  const [editingItemId, setEditingItemId] = useState<number | null>(null)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<Record<string, unknown> | null>(null)
  const [expandedItemId, setExpandedItemId] = useState<number | null>(null)

  const fieldsByKey = useMemo(
    () => new Map(nested.fields.map((f) => [f.key, compactFieldLabel(f)])),
    [nested.fields],
  )

  const { data: page, isLoading } = useQuery({
    queryKey: [path, 'list'],
    queryFn: () => api.listPage({ pageSize: 50 }),
  })
  const rows = page?.items ?? []

  useEffect(() => {
    if (editingItemId != null) return
    const next = rows.length + 1
    const current = form.getValues('numero_item')
    if (current === '' || current === String(Math.max(0, next - 1))) {
      form.setValue('numero_item', String(next))
    }
  }, [rows.length, form, editingItemId])

  const quantidade = watch('quantidade')
  const valorUnitario = watch('valor_unitario')
  const valorDesconto = watch('valor_desconto')

  useEffect(() => {
    const q = Number(quantidade)
    const u = Number(valorUnitario)
    const d = Number(valorDesconto)
    if (!Number.isFinite(q) || !Number.isFinite(u)) return
    const gross = Math.max(0, q * u - (Number.isFinite(d) ? d : 0))
    const current = form.getValues('valor_total_bruto')
    const next = String(gross)
    if (current !== next) {
      setValue('valor_total_bruto', roundMoney(gross).toFixed(2), { shouldDirty: true })
    }
  }, [quantidade, valorUnitario, valorDesconto, form, setValue])

  const createMutation = useMutation({
    mutationFn: (values: Record<string, string>) => {
      const body = buildNestedCreateBody(nested, values)
      return api.create(body)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [path] })
      reset(emptyForm(nested.fields))
      setSubmitError(null)
      setShowAddForm(false)
      toastSuccess('Item adicionado')
    },
    onError: (err) => {
      const msg = err instanceof Error ? err.message : 'Falha ao enviar'
      setSubmitError(msg)
      toastFromError(err, 'Não foi possível adicionar o item')
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, values }: { id: number; values: Record<string, string> }) => {
      const body = buildNestedCreateBody(nested, values)
      return api.update(id, body)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [path] })
      setEditingItemId(null)
      reset(emptyForm(nested.fields))
      setSubmitError(null)
      toastSuccess('Item atualizado')
    },
    onError: (err) => {
      const msg = err instanceof Error ? err.message : 'Falha ao salvar'
      setSubmitError(msg)
      toastFromError(err, 'Não foi possível atualizar o item')
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: number) => api.remove(id),
    onSuccess: (_data, id) => {
      queryClient.invalidateQueries({ queryKey: [path] })
      setDeleteTarget(null)
      if (editingItemId === id) {
        setEditingItemId(null)
        reset(emptyForm(nested.fields))
      }
      toastSuccess('Item excluído')
    },
    onError: (err) => toastFromError(err, 'Não foi possível excluir'),
  })

  const cancelEdit = useCallback(() => {
    setEditingItemId(null)
    reset(emptyForm(nested.fields))
    setSubmitError(null)
  }, [reset, nested.fields])

  const startEdit = useCallback(
    (row: Record<string, unknown>) => {
      const id = Number(row.id)
      if (!Number.isFinite(id)) return
      setShowAddForm(false)
      setExpandedItemId(null)
      setEditingItemId(id)
      reset(rowToForm(row, nested.fields))
      setSubmitError(null)
    },
    [reset, nested.fields],
  )

  const onAdd = useCallback(() => {
    void handleSubmit((values) => createMutation.mutate(values))()
  }, [handleSubmit, createMutation])

  const onSaveEdit = useCallback(() => {
    if (editingItemId == null) return
    void handleSubmit((values) => updateMutation.mutate({ id: editingItemId, values }))()
  }, [handleSubmit, updateMutation, editingItemId])

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p className="text-sm text-muted-foreground">
          {rows.length === 0
            ? 'Nenhum item. Adicione linhas de serviço para compor a DPS.'
            : `${rows.length} item(ns) na nota`}
        </p>
        <Button
          type="button"
          variant={showAddForm ? 'secondary' : 'default'}
          size="sm"
          onClick={() => {
            if (showAddForm) {
              setShowAddForm(false)
              reset(emptyForm(nested.fields))
            } else {
              cancelEdit()
              setShowAddForm(true)
            }
          }}
        >
          {showAddForm ? 'Fechar formulário' : 'Adicionar item'}
        </Button>
      </div>

      {showAddForm && editingItemId == null && (
        <div className="rounded-lg border border-dashed bg-muted/20 p-3 md:p-4">
          {nested.createHint && (
            <p className="mb-3 text-xs text-muted-foreground">{nested.createHint}</p>
          )}
          <ItemFieldsGrid
            fieldsByKey={fieldsByKey}
            form={form}
            idPrefix="item-add"
            relationScopeValues={relationScopeValues}
          />
          <div className="mt-3 flex flex-wrap items-center gap-2">
            {submitError && <p className="text-sm text-destructive">{submitError}</p>}
            <Button type="button" size="sm" disabled={createMutation.isPending} onClick={onAdd}>
              {nested.createButtonLabel ?? 'Salvar item'}
            </Button>
            <Button
              type="button"
              size="sm"
              variant="ghost"
              onClick={() => {
                reset(emptyForm(nested.fields))
                setShowAddForm(false)
              }}
            >
              Cancelar
            </Button>
          </div>
        </div>
      )}

      {isLoading ? (
        <p className="text-sm text-muted-foreground">Carregando itens…</p>
      ) : rows.length > 0 ? (
        <div className="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-12">Nº</TableHead>
                <TableHead>Descrição</TableHead>
                <TableHead className="w-24 text-right">Qtd.</TableHead>
                <TableHead className="w-28 text-right">Total</TableHead>
                <TableHead className="w-44 text-right">Ações</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {rows.map((r) => {
                const id = Number(r.id)
                const desc =
                  String(r.descricao_servico ?? r.descricao ?? '').trim() || '—'
                const expanded = expandedItemId === id
                const isEditing = editingItemId === id
                return (
                  <Fragment key={String(r.id)}>
                    <TableRow className={isEditing ? 'bg-muted/40' : undefined}>
                      <TableCell className="font-medium">{String(r.numero_item ?? '—')}</TableCell>
                      <TableCell className="max-w-[240px]" title={desc}>
                        {truncate(desc, 80)}
                      </TableCell>
                      <TableCell className="text-right">{String(r.quantidade ?? '—')}</TableCell>
                      <TableCell className="text-right">
                        {formatMoneyForDisplay(normalizeMoneyToForm(r.valor_total_bruto)) || '—'}
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-1">
                          {Number.isFinite(id) && (
                            <Button
                              type="button"
                              variant="ghost"
                              size="sm"
                              className="h-8 px-2 text-xs"
                              disabled={isEditing}
                              onClick={() => startEdit(r)}
                            >
                              Editar
                            </Button>
                          )}
                          {nested.itemTributos && Number.isFinite(id) && (
                            <Button
                              type="button"
                              variant="ghost"
                              size="sm"
                              className="h-8 px-2 text-xs"
                              disabled={isEditing}
                              onClick={() => {
                                cancelEdit()
                                setExpandedItemId(expanded ? null : id)
                              }}
                            >
                              {expanded ? 'Ocultar trib.' : 'Tributos'}
                            </Button>
                          )}
                          {Number.isFinite(id) && (
                            <Button
                              type="button"
                              variant="ghost"
                              size="sm"
                              className="h-8 px-2 text-xs text-destructive hover:text-destructive"
                              disabled={isEditing}
                              onClick={() => setDeleteTarget(r)}
                            >
                              Excluir
                            </Button>
                          )}
                        </div>
                      </TableCell>
                    </TableRow>
                    {isEditing && (
                      <TableRow className="bg-muted/20 hover:bg-muted/20">
                        <TableCell colSpan={5} className="py-3">
                          <p className="mb-3 text-sm font-medium">
                            Editar item {String(r.numero_item ?? id)}
                          </p>
                          <ItemFieldsGrid
                            fieldsByKey={fieldsByKey}
                            form={form}
                            idPrefix={`item-edit-${id}`}
                            relationScopeValues={relationScopeValues}
                          />
                          <div className="mt-3 flex flex-wrap items-center gap-2">
                            {submitError && (
                              <p className="text-sm text-destructive">{submitError}</p>
                            )}
                            <Button
                              type="button"
                              size="sm"
                              disabled={updateMutation.isPending}
                              onClick={onSaveEdit}
                            >
                              Salvar alterações
                            </Button>
                            <Button type="button" size="sm" variant="ghost" onClick={cancelEdit}>
                              Cancelar
                            </Button>
                          </div>
                        </TableCell>
                      </TableRow>
                    )}
                    {expanded && nested.itemTributos && Number.isFinite(id) && !isEditing && (
                      <TableRow className="bg-muted/30 hover:bg-muted/30">
                        <TableCell colSpan={5} className="py-2">
                          <ItemTributosPanel
                            itemId={id}
                            itemLabel={`Item ${String(r.numero_item)}`}
                            def={nested.itemTributos}
                          />
                        </TableCell>
                      </TableRow>
                    )}
                  </Fragment>
                )
              })}
            </TableBody>
          </Table>
        </div>
      ) : null}

      <DeleteConfirmDialog
        open={deleteTarget != null}
        onOpenChange={(open) => {
          if (!open) setDeleteTarget(null)
        }}
        title="Excluir item?"
        description={
          deleteTarget
            ? `Item ${String(deleteTarget.numero_item ?? deleteTarget.id)} — esta ação não pode ser desfeita.`
            : ''
        }
        pending={deleteMutation.isPending}
        onConfirm={() => {
          if (deleteTarget?.id) deleteMutation.mutate(Number(deleteTarget.id))
        }}
      />
    </div>
  )
}
