import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

function formatMoney(value: unknown): string {
  if (value == null || value === '') return '—'
  const n = Number(value)
  if (!Number.isFinite(n)) return String(value)
  return n.toLocaleString('pt-BR', { style: 'currency', currency: 'BRL' })
}

const VALOR_FIELDS = [
  {
    key: 'valor_total_servicos_produtos',
    label: 'Valor da operação/serviço prestado',
  },
  { key: 'valor_recebido_intermediario', label: 'Valor recebido pelo intermediário' },
  { key: 'desconto_incondicionado', label: 'Desconto incondicionado' },
  { key: 'desconto_condicionado', label: 'Desconto condicionado' },
  { key: 'valor_liquido', label: 'Valor líquido' },
] as const

export function DocumentoValoresShowCard({ row }: { row: Record<string, unknown> }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Valores do Fornecimento/Serviço Prestado</CardTitle>
      </CardHeader>
      <CardContent>
        <dl className="grid gap-3 sm:grid-cols-2">
          {VALOR_FIELDS.map((field) => (
            <div key={field.key}>
              <dt className="text-sm font-medium text-muted-foreground">{field.label}</dt>
              <dd className="text-sm tabular-nums">{formatMoney(row[field.key])}</dd>
            </div>
          ))}
        </dl>
      </CardContent>
    </Card>
  )
}
