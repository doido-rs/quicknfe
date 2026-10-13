import { apiFetch, createApiError } from './client'

const API_BASE = import.meta.env.VITE_API_URL ?? ''

function filenameFromContentDisposition(header: string | null): string | null {
  if (!header) return null
  const utf8 = header.match(/filename\*=UTF-8''([^;]+)/i)
  if (utf8?.[1]) {
    try {
      return decodeURIComponent(utf8[1])
    } catch {
      return utf8[1]
    }
  }
  const plain = header.match(/filename="([^"]+)"/i) ?? header.match(/filename=([^;]+)/i)
  return plain?.[1]?.trim() ?? null
}

export function documentoHasXml(_row?: Record<string, unknown>): boolean {
  return true
}

export function documentoXmlAssinado(row: Record<string, unknown>): boolean {
  const signed = row.xml_assinado
  return typeof signed === 'string' && signed.includes('Signature')
}

export async function assinarDocumento(documentoId: number): Promise<Record<string, unknown>> {
  return apiFetch<Record<string, unknown>>(`/api/documentos/${documentoId}/assinar`, {
    method: 'POST',
    body: JSON.stringify({}),
  })
}

export async function downloadDocumentoXml(documentoId: number): Promise<void> {
  const res = await fetch(`${API_BASE}/api/documentos/${documentoId}/xml`, {
    credentials: 'include',
  })

  if (!res.ok) {
    const text = await res.text()
    throw createApiError(res.status, text || res.statusText)
  }

  const blob = await res.blob()
  const filename =
    filenameFromContentDisposition(res.headers.get('Content-Disposition')) ??
    `nota-fiscal-${documentoId}.xml`

  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  anchor.rel = 'noopener'
  document.body.appendChild(anchor)
  anchor.click()
  anchor.remove()
  URL.revokeObjectURL(url)
}
