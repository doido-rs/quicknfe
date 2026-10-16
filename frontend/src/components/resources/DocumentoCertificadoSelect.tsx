import { useQuery } from '@tanstack/react-query'
import { resourceApi } from '@/api/client'
import { Label } from '@/components/ui/label'

export type CertificadoRow = {
  id: number
  tipo: string
  data_validade?: string
}

export function usePrestadorCertificados(empresaId: number | undefined, enabled: boolean) {
  const certPath =
    empresaId != null && Number.isFinite(empresaId) && empresaId > 0
      ? `/api/prestadores/${empresaId}/certificados`
      : null

  const query = useQuery({
    queryKey: [certPath, 'list'],
    queryFn: () => resourceApi<CertificadoRow>(certPath!).listPage({ pageSize: 20 }),
    enabled: certPath != null && enabled,
  })

  return {
    certs: query.data?.items ?? [],
    certPath,
  }
}

export function DocumentoCertificadoSelect({
  empresaId,
  enabled,
  value,
  onChange,
  labelId,
  label = 'Certificado (mTLS / assinatura)',
}: {
  empresaId: number | undefined
  enabled: boolean
  value: string
  onChange: (value: string) => void
  labelId: string
  label?: string
}) {
  const { certs } = usePrestadorCertificados(empresaId, enabled)

  if (!enabled || certs.length <= 1) return null

  return (
    <div className="flex min-w-[12rem] flex-col gap-1">
      <Label htmlFor={labelId} className="text-xs text-muted-foreground">
        {label}
      </Label>
      <select
        id={labelId}
        className="h-8 rounded-md border border-input bg-background px-2 text-xs"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      >
        <option value="auto">Padrão (mais recente)</option>
        {certs.map((c) => (
          <option key={c.id} value={String(c.id)}>
            {c.tipo} · #{c.id}
          </option>
        ))}
      </select>
    </div>
  )
}

export function certificadoIdFromSelect(value: string): number | undefined {
  return value === 'auto' ? undefined : Number(value)
}
