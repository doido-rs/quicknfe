import { useQuery } from '@tanstack/react-query'
import { resourceApi } from '@/api/client'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { formatMunicipioRelationLabel } from '@/lib/resourceForm'

function MunicipioLine({ municipioId }: { municipioId: unknown }) {
  const id = Number(municipioId)
  const api = resourceApi<Record<string, unknown>>('/api/municipios')
  const { data } = useQuery({
    queryKey: ['municipios', 'detail', id],
    queryFn: () => api.get(id),
    enabled: Number.isFinite(id) && id > 0,
  })

  if (!Number.isFinite(id) || id <= 0) {
    return <>—</>
  }

  const label = data ? formatMunicipioRelationLabel(data) : null
  return <>{label ?? `ID ${id}`}</>
}

export function DocumentoMunicipiosShowCard({ row }: { row: Record<string, unknown> }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Municípios da nota</CardTitle>
      </CardHeader>
      <CardContent>
        <dl className="grid gap-3 sm:grid-cols-2">
          <div>
            <dt className="text-sm font-medium text-muted-foreground">Emissão da nota</dt>
            <dd className="text-sm">
              <MunicipioLine municipioId={row.municipio_emissao_id} />
            </dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-muted-foreground">Prestação do serviço</dt>
            <dd className="text-sm">
              <MunicipioLine municipioId={row.municipio_prestacao_id} />
            </dd>
          </div>
        </dl>
      </CardContent>
    </Card>
  )
}
