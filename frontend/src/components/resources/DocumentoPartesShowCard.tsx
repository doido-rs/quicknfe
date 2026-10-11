import {
  DOCUMENTO_PARTES,
  localidadeParteLabel,
  type ParteLocalidadeFormDef,
} from '@/lib/documentoParteLocalidade'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { formatFieldValue } from '@/lib/resourceForm'

const EXTERIOR_LABELS: Record<string, string> = {
  razao_social: 'Nome/Razão Social',
  telefone: 'Telefone',
  email: 'E-mail',
  logradouro: 'Logradouro',
  numero: 'Número',
  complemento: 'Complemento',
  bairro: 'Bairro',
  cidade: 'Cidade',
  codigo_postal: 'Código de Endereçamento Postal',
  estado_regiao: 'Estado, província ou região',
  pais: 'País',
}

function ParteShow({
  parte,
  row,
}: {
  parte: ParteLocalidadeFormDef
  row: Record<string, unknown>
}) {
  const localidade = String(row[parte.localidadeFieldKey] ?? 'nao_informado')
  const exterior = row[parte.exteriorPrefix]

  return (
    <div className="space-y-2 rounded-md border border-dashed p-4">
      <h3 className="text-sm font-semibold">{parte.title}</h3>
      <dl className="grid gap-2 sm:grid-cols-2">
        <div>
          <dt className="text-sm text-muted-foreground">Localidade</dt>
          <dd className="text-sm">{localidadeParteLabel(localidade)}</dd>
        </div>
        {localidade === 'brasil' && (
          <div>
            <dt className="text-sm text-muted-foreground">Cadastro (Brasil)</dt>
            <dd className="text-sm">
              {formatFieldValue(row[parte.brasilRelationFieldKey], {
                key: parte.brasilRelationFieldKey,
                label: parte.title,
                relationResource:
                  parte.brasilRelationFieldKey === 'cliente_id' ? 'clientes' : 'intermediarios',
              })}
            </dd>
          </div>
        )}
      </dl>
      {localidade === 'exterior' && exterior != null && typeof exterior === 'object' && (
        <dl className="mt-2 grid gap-2 sm:grid-cols-2">
          {Object.entries(EXTERIOR_LABELS).map(([key, label]) => (
            <div key={key}>
              <dt className="text-sm text-muted-foreground">{label}</dt>
              <dd className="text-sm">
                {formatFieldValue((exterior as Record<string, unknown>)[key])}
              </dd>
            </div>
          ))}
        </dl>
      )}
    </div>
  )
}

export function DocumentoPartesShowCard({ row }: { row: Record<string, unknown> }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Tomador e intermediário</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {DOCUMENTO_PARTES.map((parte) => (
          <ParteShow key={parte.localidadeFieldKey} parte={parte} row={row} />
        ))}
      </CardContent>
    </Card>
  )
}
