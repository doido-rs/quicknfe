import { useMutation, useQueryClient } from '@tanstack/react-query'
import { PenLine } from 'lucide-react'
import { assinarDocumento, documentoXmlAssinado } from '@/api/documentos'
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

  const mutation = useMutation({
    mutationFn: () => assinarDocumento(documentoId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documentos', 'detail', documentoId] })
      queryClient.invalidateQueries({ queryKey: ['documentos', 'list'] })
      toastSuccess('Nota fiscal assinada')
    },
    onError: (err) => toastFromError(err, 'Não foi possível assinar a nota'),
  })

  return (
    <Button
      type="button"
      variant="default"
      className="gap-2"
      disabled={mutation.isPending}
      onClick={() => mutation.mutate()}
    >
      <PenLine className="size-4" />
      {mutation.isPending
        ? 'Assinando…'
        : alreadySigned
          ? 'Assinar novamente'
          : 'Assinar nota fiscal'}
    </Button>
  )
}
