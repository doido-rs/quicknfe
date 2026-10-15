import { useQueryClient } from '@tanstack/react-query'
import { useEffect } from 'react'
import { resourceApi } from '@/api/client'
import {
  documentoProcessamentoInFlight,
  documentoTerminalError,
} from '@/lib/documentoProcessamento'
import { toastFromError, toastSuccess } from '@/lib/toast'

export function useDocumentoProcessamentoPoll(
  documentoId: number,
  row: Record<string, unknown> | undefined,
) {
  const queryClient = useQueryClient()
  const status = row?.status
  const inFlight = documentoProcessamentoInFlight(status)

  useEffect(() => {
    if (!inFlight || !Number.isFinite(documentoId)) return

    const api = resourceApi<Record<string, unknown>>('/api/documentos')
    let cancelled = false

    const tick = async () => {
      try {
        const fresh = await api.get(documentoId)
        if (cancelled) return
        queryClient.setQueryData(['documentos', 'detail', documentoId], fresh)
        const s = String(fresh.status ?? '')
        if (!documentoProcessamentoInFlight(s)) {
          queryClient.invalidateQueries({ queryKey: ['documentos', 'list'] })
          if (documentoTerminalError(s)) {
            const msg =
              typeof fresh.processamento_mensagem === 'string'
                ? fresh.processamento_mensagem
                : 'Processamento da nota falhou'
            toastFromError(new Error(msg), msg)
          } else if (s === 'assinado' || s === 'autorizada' || s === 'autorizado') {
            toastSuccess(
              s === 'assinado' ? 'Nota fiscal assinada' : 'Nota fiscal transmitida à SEFIN',
            )
          }
        }
      } catch {
        /* ignore transient poll errors */
      }
    }

    void tick()
    const id = window.setInterval(() => void tick(), 2500)
    return () => {
      cancelled = true
      window.clearInterval(id)
    }
  }, [documentoId, inFlight, queryClient])
}
