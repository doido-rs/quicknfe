import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Send } from 'lucide-react'
import { documentoXmlAssinado, transmitirDocumento } from '@/api/documentos'
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

  const mutation = useMutation({
    mutationFn: () => transmitirDocumento(documentoId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documentos', 'detail', documentoId] })
      queryClient.invalidateQueries({ queryKey: ['documentos', 'list'] })
      toastSuccess('Nota fiscal transmitida à SEFIN')
    },
    onError: (err) => toastFromError(err, 'Não foi possível transmitir a nota'),
  })

  return (
    <Button
      type="button"
      variant="secondary"
      className="gap-2"
      disabled={!signed || autorizada || mutation.isPending}
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
      {mutation.isPending ? 'Transmitindo…' : 'Transmitir SEFIN'}
    </Button>
  )
}
