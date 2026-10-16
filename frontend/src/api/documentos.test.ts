import { afterEach, describe, expect, it, vi } from 'vitest'
import * as client from './client'
import {
  assinarDocumento,
  documentoHasXml,
  documentoXmlAssinado,
  downloadDocumentoXml,
  transmitirDocumento,
} from './documentos'

afterEach(() => {
  vi.restoreAllMocks()
})

describe('documento helpers', () => {
  it('documentoHasXml is always true', () => {
    expect(documentoHasXml()).toBe(true)
    expect(documentoHasXml({})).toBe(true)
  })

  it('documentoXmlAssinado detects Signature in xml_assinado', () => {
    expect(documentoXmlAssinado({ xml_assinado: '<root><Signature/></root>' })).toBe(true)
    expect(documentoXmlAssinado({ xml_assinado: 'draft' })).toBe(false)
    expect(documentoXmlAssinado({})).toBe(false)
  })
})

describe('documento API', () => {
  it('assinarDocumento posts optional certificado_id', async () => {
    vi.spyOn(client, 'apiFetch').mockResolvedValue({ documento_id: 1, status: 'assinando' })
    await assinarDocumento(9, 3)
    expect(client.apiFetch).toHaveBeenCalledWith('/api/documentos/9/assinar', {
      method: 'POST',
      body: JSON.stringify({ certificado_id: 3 }),
    })
    await assinarDocumento(9)
    expect(client.apiFetch).toHaveBeenLastCalledWith('/api/documentos/9/assinar', {
      method: 'POST',
      body: JSON.stringify({}),
    })
  })

  it('transmitirDocumento posts to transmitir', async () => {
    vi.spyOn(client, 'apiFetch').mockResolvedValue({ documento_id: 2, status: 'transmitindo' })
    await transmitirDocumento(2)
    expect(client.apiFetch).toHaveBeenCalledWith('/api/documentos/2/transmitir', {
      method: 'POST',
      body: JSON.stringify({}),
    })
  })

  it('downloadDocumentoXml saves blob with filename from header', async () => {
    const click = vi.fn()
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        headers: {
          get: (name: string) =>
            name.toLowerCase() === 'content-disposition'
              ? 'attachment; filename="nota-42.xml"'
              : null,
        },
        blob: async () => new Blob(['<xml/>'], { type: 'application/xml' }),
      }),
    )
    vi.stubGlobal('URL', {
      createObjectURL: vi.fn(() => 'blob:mock'),
      revokeObjectURL: vi.fn(),
    })
    const anchor = { click, remove: vi.fn() } as unknown as HTMLAnchorElement
    vi.spyOn(document, 'createElement').mockReturnValue(anchor)
    vi.spyOn(document.body, 'appendChild').mockImplementation(() => anchor)

    await downloadDocumentoXml(42)

    expect(click).toHaveBeenCalled()
    expect(anchor.download).toBe('nota-42.xml')
  })

  it('downloadDocumentoXml throws api error on failure', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: false,
        status: 404,
        text: async () => 'missing',
      }),
    )
    await expect(downloadDocumentoXml(1)).rejects.toSatisfy(client.isApiError)
  })
})
