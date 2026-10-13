import { useEffect, useState } from 'react'
import { ResourceFieldInput } from '@/components/resources/ResourceFieldInput'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  buildNestedCreateBody,
  emptyForm,
  rowLabel,
  useResourceFieldForm,
} from '@/lib/resourceForm'
import type { NestedDef } from '@/resources/definitions'

export function DocumentoItensDraftPanel({
  nested,
  items,
  onItemsChange,
}: {
  nested: NestedDef
  items: Record<string, unknown>[]
  onItemsChange: (items: Record<string, unknown>[]) => void
}) {
  const form = useResourceFieldForm(nested.fields)
  const { handleSubmit, reset } = form
  const [submitError, setSubmitError] = useState<string | null>(null)

  useEffect(() => {
    if (!nested.fields.some((f) => f.key === 'numero_item')) return
    const next = items.length + 1
    form.setValue('numero_item', String(next))
  }, [items.length, nested.fields, form])

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{nested.label}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <form
          className="grid gap-4 md:grid-cols-2"
          onSubmit={handleSubmit((values) => {
            setSubmitError(null)
            const servico = values.descricao_servico?.trim() ?? ''
            if (!servico) {
              setSubmitError('Informe a descrição do serviço.')
              return
            }
            if (!values.cna_id?.trim()) {
              setSubmitError('Selecione o código de tributação nacional (CNA).')
              return
            }
            const body = buildNestedCreateBody(nested, values)
            onItemsChange([...items, body])
            reset(emptyForm(nested.fields))
          })}
        >
          {nested.fields.map((f) => (
            <ResourceFieldInput
              key={f.key}
              field={f}
              form={form}
              idPrefix={`draft-item-${f.key}`}
            />
          ))}
          <div className="md:col-span-2">
            {submitError && <p className="mb-2 text-sm text-destructive">{submitError}</p>}
            <Button type="submit">{nested.createButtonLabel ?? 'Adicionar item'}</Button>
          </div>
        </form>

        {items.length > 0 ? (
          <ul className="space-y-2 text-sm">
            {items.map((item, index) => (
              <li
                key={`${String(item.numero_item)}-${index}`}
                className="flex flex-wrap items-start justify-between gap-2 rounded-md border px-3 py-2"
              >
                <span className="min-w-0 flex-1">{rowLabel(item, nested.fields)}</span>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="shrink-0 text-destructive hover:text-destructive"
                  onClick={() => onItemsChange(items.filter((_, i) => i !== index))}
                >
                  Remover
                </Button>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-muted-foreground">
            Nenhum item adicionado. Você pode incluir itens agora; eles serão salvos ao criar a
            nota.
          </p>
        )}
      </CardContent>
    </Card>
  )
}
