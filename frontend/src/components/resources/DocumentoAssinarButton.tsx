import { useMutation, useQueryClient } from '@tanstack/react-query'
import { PenLine } from 'lucide-react'
import { useState } from 'react'
import { assinarDocumento, documentoXmlAssinado } from '@/api/documentos'
import {
  certificadoIdFromSelect,
  DocumentoCertificadoSelect,
} from '@/components/resources/DocumentoCertificadoSelect'
import { documentoProcessamentoInFlight } from '@/lib/documentoProcessamento'
import { Button } from '@/components/ui/button'
import { toastFromError, toastSuccess } from '@/lib/toast'

export function DocumentoAssinarButton({
  documentoId,
  row,
}: {
  documentoId: number
  row: Record<string, unknown>
}) {
  const queryClient = useQueryClient()
  const alreadySigned = documentoXmlAssinado(row)
  const inFlight = documentoProcessamentoInFlight(row.status)
  const empresaId = Number(row.empresa_id)
  const [certificadoId, setCertificadoId] = useState<string>('auto')

  const mutation = useMutation({
    mutationFn: () =>
      assinarDocumento(documentoId, certificadoIdFromSelect(certificadoId)),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documentos', 'detail', documentoId] })
      queryClient.invalidateQueries({ queryKey: ['documentos', 'list'] })
      toastSuccess('Assinatura enfileirada')
    },
    onError: (err) => toastFromError(err, 'Não foi possível assinar a nota'),
  })

  return (
    <div className="flex flex-col items-end gap-2">
      <DocumentoCertificadoSelect
        empresaId={empresaId}
        enabled={!inFlight && !mutation.isPending}
        value={certificadoId}
        onChange={setCertificadoId}
        labelId="assinar-cert"
        label="Certificado (assinatura)"
      />
      <Button
        type="button"
        variant="default"
        className="gap-2"
        disabled={mutation.isPending || inFlight}
        onClick={() => mutation.mutate()}
      >
        <PenLine className="size-4" />
        {mutation.isPending || inFlight
          ? 'Assinando…'
          : alreadySigned
            ? 'Assinar novamente'
            : 'Assinar nota fiscal'}
      </Button>
    </div>
  )
}
