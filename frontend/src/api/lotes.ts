import { apiFetch } from './client'
import type { DocumentoEnqueueResponse } from './documentos'

export async function transmitirLote(loteId: number): Promise<DocumentoEnqueueResponse[]> {
  return apiFetch<DocumentoEnqueueResponse[]>(`/api/lotes/${loteId}/transmitir`, {
    method: 'POST',
    body: JSON.stringify({}),
  })
}
