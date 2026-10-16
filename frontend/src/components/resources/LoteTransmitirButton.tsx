import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Send } from 'lucide-react'
import { transmitirLote } from '@/api/lotes'
import { Button } from '@/components/ui/button'
import { toastFromError, toastSuccess } from '@/lib/toast'

export function LoteTransmitirButton({ loteId }: { loteId: number }) {
  const queryClient = useQueryClient()

  const mutation = useMutation({
    mutationFn: () => transmitirLote(loteId),
    onSuccess: (results) => {
      queryClient.invalidateQueries({ queryKey: ['lotes'] })
      queryClient.invalidateQueries({ queryKey: ['documentos'] })
      const failures = results.filter(
        (r) =>
          typeof r.processamento_mensagem === 'string' &&
          r.processamento_mensagem.trim() !== '' &&
          !String(r.status ?? '').toLowerCase().includes('transmitindo'),
      )
      if (failures.length > 0) {
        toastFromError(
          new Error(failures.map((f) => f.processamento_mensagem).join('; ')),
          `Transmissão enfileirada com ${failures.length} aviso(s)`,
        )
      } else {
        toastSuccess(
          results.length === 1
            ? 'Transmissão enfileirada para 1 nota'
            : `Transmissão enfileirada para ${results.length} notas`,
        )
      }
    },
    onError: (err) => toastFromError(err, 'Não foi possível transmitir o lote'),
  })

  return (
    <Button
      type="button"
      variant="secondary"
      className="gap-2"
      disabled={mutation.isPending}
      onClick={() => mutation.mutate()}
    >
      <Send className="size-4" />
      {mutation.isPending ? 'Enfileirando…' : 'Transmitir lote'}
    </Button>
  )
}
