import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Send } from 'lucide-react'
import { useState } from 'react'
import { documentoXmlAssinado, transmitirDocumento } from '@/api/documentos'
import {
  certificadoIdFromSelect,
  DocumentoCertificadoSelect,
} from '@/components/resources/DocumentoCertificadoSelect'
import { documentoProcessamentoInFlight } from '@/lib/documentoProcessamento'
import { Button } from '@/components/ui/button'
import { toastFromError, toastSuccess } from '@/lib/toast'

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
  const [certificadoId, setCertificadoId] = useState<string>('auto')

  const mutation = useMutation({
    mutationFn: () =>
      transmitirDocumento(documentoId, certificadoIdFromSelect(certificadoId)),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documentos', 'detail', documentoId] })
      queryClient.invalidateQueries({ queryKey: ['documentos', 'list'] })
      toastSuccess('Transmissão enfileirada')
    },
    onError: (err) => toastFromError(err, 'Não foi possível transmitir a nota'),
  })

  return (
    <div className="flex flex-col items-end gap-2">
      <DocumentoCertificadoSelect
        empresaId={empresaId}
        enabled={signed && !autorizada && !inFlight && !mutation.isPending}
        value={certificadoId}
        onChange={setCertificadoId}
        labelId="transmit-cert"
        label="Certificado (mTLS)"
      />
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
