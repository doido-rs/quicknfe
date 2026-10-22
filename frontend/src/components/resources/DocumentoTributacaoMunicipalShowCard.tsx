import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  regimeEspecialTribIssqnLabel,
  simNaoLabel,
  tributacaoIssqnLabel,
} from '@/lib/documentoTributacaoIssqn'
import { formatPercentValueForDisplay } from '@/lib/percentFormat'

function formatMoney(value: unknown): string {
  if (value == null || value === '') return '—'
  const n = Number(value)
  if (!Number.isFinite(n)) return String(value)
  return n.toLocaleString('pt-BR', { style: 'currency', currency: 'BRL' })
}

export function DocumentoTributacaoMunicipalShowCard({ row }: { row: Record<string, unknown> }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Tributação Municipal</CardTitle>
      </CardHeader>
      <CardContent>
        <dl className="grid gap-3 sm:grid-cols-2">
          <div className="sm:col-span-2">
            <dt className="text-sm font-medium text-muted-foreground">
              Tributação do ISSQN sobre o serviço prestado
            </dt>
            <dd className="text-sm">{tributacaoIssqnLabel(row.trib_issqn)}</dd>
          </div>
          <div className="sm:col-span-2">
            <dt className="text-sm font-medium text-muted-foreground">
              Regime Especial de Tributação
            </dt>
            <dd className="text-sm">{regimeEspecialTribIssqnLabel(row.reg_esp_trib_issqn)}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-muted-foreground">
              Exigibilidade do recolhimento suspensa
            </dt>
            <dd className="text-sm">{simNaoLabel(row.issqn_exigibilidade_suspensa)}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-muted-foreground">
              Retenção pelo tomador ou intermediário
            </dt>
            <dd className="text-sm">{simNaoLabel(row.issqn_retencao)}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-muted-foreground">Benefício municipal</dt>
            <dd className="text-sm">{simNaoLabel(row.issqn_beneficio_municipal)}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-muted-foreground">
              Dedução/redução da base de cálculo
            </dt>
            <dd className="text-sm">{simNaoLabel(row.issqn_deducao_reducao_bc)}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-muted-foreground">Alíquota</dt>
            <dd className="text-sm tabular-nums">
              {formatPercentValueForDisplay(row.issqn_aliquota)}
            </dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-muted-foreground">BC ISSQN</dt>
            <dd className="text-sm tabular-nums">{formatMoney(row.issqn_base_calculo)}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-muted-foreground">Valor ISSQN</dt>
            <dd className="text-sm tabular-nums">{formatMoney(row.issqn_valor)}</dd>
          </div>
        </dl>
      </CardContent>
    </Card>
  )
}
