import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { formatFieldValue } from '@/lib/resourceForm'

const INFO_COMPL_FIELDS = [
  {
    key: 'numero_doc_responsabilidade_tecnica',
    label: 'Número do documento de responsabilidade técnica',
  },
  { key: 'documento_referencia', label: 'Documento de referência' },
  { key: 'informacoes_complementares', label: 'Informações complementares' },
  {
    key: 'numero_pedido_ordem_b2b',
    label:
      'Número do Pedido, Ordem de Compra, Ordem de Serviço ou Projeto que autorize a prestação do serviço/fornecimento em operações B2B',
  },
] as const

function hasAnyValue(row: Record<string, unknown>): boolean {
  return INFO_COMPL_FIELDS.some((f) => {
    const v = row[f.key]
    return v != null && String(v).trim() !== ''
  })
}

export function DocumentoInformacoesComplementaresShowCard({
  row,
}: {
  row: Record<string, unknown>
}) {
  if (!hasAnyValue(row)) {
    return null
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Informações complementares</CardTitle>
      </CardHeader>
      <CardContent>
        <dl className="grid gap-4 sm:grid-cols-2">
          {INFO_COMPL_FIELDS.map((field) => {
            const raw = row[field.key]
            const isLong =
              field.key === 'informacoes_complementares' || field.key === 'numero_pedido_ordem_b2b'
            return (
              <div key={field.key} className={isLong ? 'sm:col-span-2' : undefined}>
                <dt className="text-sm font-medium text-muted-foreground">{field.label}</dt>
                <dd className="mt-1 whitespace-pre-wrap text-sm">
                  {formatFieldValue(raw)}
                </dd>
              </div>
            )
          })}
        </dl>
      </CardContent>
    </Card>
  )
}
