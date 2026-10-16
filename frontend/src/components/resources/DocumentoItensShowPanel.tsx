import { useQuery } from '@tanstack/react-query'
import { resourceApi } from '@/api/client'
import { ItemTributosPanel } from '@/components/resources/ItemTributosPanel'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { readOnlyFieldDisplay } from '@/lib/documentoShowLabels'
import { rowToForm } from '@/lib/resourceForm'
import { useDateFormat } from '@/theme/DateFormatContext'
import { documentoItemShowFields, type NestedDef } from '@/resources/definitions'

function itemCnaLabel(row: Record<string, unknown>): string | undefined {
  const codigo = row.codigo_tributacao_nacional
  if (codigo != null && String(codigo).trim() !== '') {
    return String(codigo)
  }
  const id = row.cna_id
  if (id != null && String(id).trim() !== '') {
    return `CNA #${String(id)}`
  }
  return undefined
}

export function DocumentoItensShowPanel({
  documentoId,
  nested,
}: {
  documentoId: number
  nested: NestedDef
}) {
  const path = nested.path(documentoId)
  const api = resourceApi<Record<string, unknown>>(path)
  const dateFormat = useDateFormat()
  const showFields = documentoItemShowFields(nested)

  const { data: page, isLoading } = useQuery({
    queryKey: [path, 'list'],
    queryFn: () => api.listPage({ pageSize: 100 }),
  })
  const rows = page?.items ?? []

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{nested.label}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        {isLoading && <p className="text-sm text-muted-foreground">Carregando itens…</p>}
        {!isLoading && rows.length === 0 && (
          <p className="text-sm text-muted-foreground">
            Nenhum item cadastrado. Use <strong>Editar</strong> na nota para incluir itens.
          </p>
        )}
        {rows.map((row) => {
          const formValues = rowToForm(row, showFields)
          const relationLabels: Record<string, string> = {}
          const cna = itemCnaLabel(row)
          if (cna) relationLabels.cna_id = cna

          return (
            <div
              key={String(row.id)}
              className="space-y-3 rounded-lg border border-border/70 bg-muted/10 p-4"
            >
              <p className="text-sm font-medium">
                Item {String(row.numero_item ?? '—')}
                {row.id != null ? (
                  <span className="font-normal text-muted-foreground"> · ID {String(row.id)}</span>
                ) : null}
              </p>
              <dl className="grid gap-3 sm:grid-cols-2">
                {showFields.map((field) => {
                  const text = readOnlyFieldDisplay(
                    field,
                    formValues[field.key] ?? '',
                    row,
                    relationLabels,
                    dateFormat,
                  )
                  return (
                    <div key={field.key} className={field.type === 'textarea' ? 'sm:col-span-2' : ''}>
                      <dt className="text-xs font-medium text-muted-foreground">{field.label}</dt>
                      <dd className="mt-0.5 text-sm whitespace-pre-wrap break-words">{text || '—'}</dd>
                    </div>
                  )
                })}
              </dl>
              {nested.itemTributos && Number.isFinite(Number(row.id)) && (
                <ul className="border-t border-border/60 pt-3">
                  <ItemTributosPanel
                    itemId={Number(row.id)}
                    itemLabel={`Item ${String(row.numero_item ?? row.id)}`}
                    def={nested.itemTributos}
                    readOnly
                  />
                </ul>
              )}
            </div>
          )
        })}
      </CardContent>
    </Card>
  )
}
