import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Send } from 'lucide-react'
import { useState } from 'react'
import { resourceApi } from '@/api/client'
import { documentoXmlAssinado, transmitirDocumento } from '@/api/documentos'
import { documentoProcessamentoInFlight } from '@/lib/documentoProcessamento'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { toastFromError, toastSuccess } from '@/lib/toast'

type CertificadoRow = {
  id: number
  tipo: string
  data_validade?: string
}

export function DocumentoTransmitirButton({
  documentoId,
  row,
}: {
  documentoId: number
  row: Record<string, unknown>
}) {
  const queryClient = useQueryClient()
  const signed = documentoXmlAssinado(row)
  const status = String(row.status ?? '').toLowerCase()
  const autorizada = status === 'autorizada' || status === 'autorizado'
  const inFlight = documentoProcessamentoInFlight(row.status)
  const empresaId = Number(row.empresa_id)
  const certPath =
    Number.isFinite(empresaId) && empresaId > 0
      ? `/api/prestadores/${empresaId}/certificados`
      : null

  const { data: certPage } = useQuery({
    queryKey: [certPath, 'list'],
    queryFn: () => resourceApi<CertificadoRow>(certPath!).listPage({ pageSize: 20 }),
    enabled: certPath != null && signed && !autorizada,
  })
  const certs = certPage?.items ?? []

  const [certificadoId, setCertificadoId] = useState<string>('auto')

  const mutation = useMutation({
    mutationFn: () =>
      transmitirDocumento(
        documentoId,
        certificadoId === 'auto' ? undefined : Number(certificadoId),
      ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documentos', 'detail', documentoId] })
      queryClient.invalidateQueries({ queryKey: ['documentos', 'list'] })
      toastSuccess('Transmissão enfileirada')
    },
    onError: (err) => toastFromError(err, 'Não foi possível transmitir a nota'),
  })

  const showCertSelect = signed && !autorizada && certs.length > 1

  return (
    <div className="flex flex-col items-end gap-2">
      {showCertSelect && (
        <div className="flex min-w-[12rem] flex-col gap-1">
          <Label htmlFor="transmit-cert" className="text-xs text-muted-foreground">
            Certificado (mTLS)
          </Label>
          <select
            id="transmit-cert"
            className="h-8 rounded-md border border-input bg-background px-2 text-xs"
            value={certificadoId}
            onChange={(e) => setCertificadoId(e.target.value)}
          >
            <option value="auto">Padrão (mais recente)</option>
            {certs.map((c) => (
              <option key={c.id} value={String(c.id)}>
                {c.tipo} · #{c.id}
              </option>
            ))}
          </select>
        </div>
      )}
      <Button
        type="button"
        variant="secondary"
        className="gap-2"
        disabled={!signed || autorizada || mutation.isPending || inFlight}
        title={
          !signed
            ? 'Assine a nota fiscal antes de transmitir'
            : autorizada
              ? 'Nota já autorizada'
              : undefined
        }
        onClick={() => mutation.mutate()}
      >
        <Send className="size-4" />
        {mutation.isPending || inFlight ? 'Transmitindo…' : 'Transmitir SEFIN'}
      </Button>
    </div>
  )
}
