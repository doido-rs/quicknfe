import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  cstPisCofinsLabel,
  tpRetPisCofinsCsllLabel,
} from '@/lib/documentoTributacaoFederal'

function formatMoney(value: unknown): string {
  if (value == null || value === '') return '—'
  const n = Number(value)
  if (!Number.isFinite(n)) return String(value)
  return n.toLocaleString('pt-BR', { style: 'currency', currency: 'BRL' })
}

export function DocumentoTributacaoFederalShowCard({ row }: { row: Record<string, unknown> }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Tributação Federal</CardTitle>
      </CardHeader>
      <CardContent>
        <dl className="grid gap-3 sm:grid-cols-2">
          <div className="sm:col-span-2">
            <dt className="text-sm font-medium text-muted-foreground">
              Situação Tributária do PIS/COFINS
            </dt>
            <dd className="text-sm">{cstPisCofinsLabel(row.cst_pis_cofins)}</dd>
          </div>
          <div className="sm:col-span-2">
            <dt className="text-sm font-medium text-muted-foreground">
              Tipo de retenção do PIS/COFINS/CSLL
            </dt>
            <dd className="text-sm">{tpRetPisCofinsCsllLabel(row.tp_ret_pis_cofins_csll)}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-muted-foreground">IRRF</dt>
            <dd className="text-sm tabular-nums">{formatMoney(row.valor_irrf)}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-muted-foreground">
              Contribuições Sociais — Retidas
            </dt>
            <dd className="text-sm tabular-nums">
              {formatMoney(row.valor_contrib_sociais_retidas)}
            </dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-muted-foreground">
              Contribuição Previdenciária — Retida
            </dt>
            <dd className="text-sm tabular-nums">
              {formatMoney(row.valor_contrib_previdenciaria_retida)}
            </dd>
          </div>
        </dl>
      </CardContent>
    </Card>
  )
}
