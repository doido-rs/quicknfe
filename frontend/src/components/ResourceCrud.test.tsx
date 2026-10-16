import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { asPage } from '../test/pageResponse'
import { renderWithProviders } from '../test/renderWithProviders'

function jsonResponse(data: unknown, status = 200) {
  return {
    ok: status >= 200 && status < 300,
    status,
    text: async () => '',
    json: async () => data,
  }
}

function stubFetch(handler: (url: string, method: string) => ReturnType<typeof jsonResponse>) {
  vi.stubGlobal(
    'fetch',
    vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input)
      const method = init?.method ?? 'GET'
      if (url.includes('/api/me')) {
        return jsonResponse({
          user: { id: 1, email: 'u@test.com' },
          companies: [],
          company_ativa: { id: 1, nome: 'C' },
          prestadores: [],
          prestador_ativo: { id: 1, cpf_cnpj: '1', razao_social: 'E' },
          papel: 'p',
        })
      }
      return handler(url, method)
    }),
  )
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('Resource pages', () => {
  it('lists clientes with Novo action', async () => {
    stubFetch((url, method) => {
      if (method === 'GET' && /\/api\/clientes(\?|$)/.test(url)) {
        return jsonResponse(asPage([{ id: 1, cpf_cnpj: '123', razao_social: 'Cliente A' }]))
      }
      return jsonResponse('missing', 404)
    })

    await renderWithProviders({ route: '/app/clientes' })
    expect(await screen.findByText('Cliente A')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /^novo$/i })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /^ver$/i })).toBeInTheDocument()
  })

  it('shows cliente detail', async () => {
    stubFetch((url, method) => {
      if (method === 'GET' && url.endsWith('/api/clientes/1')) {
        return jsonResponse({ id: 1, cpf_cnpj: '123', razao_social: 'Cliente A' })
      }
      if (method === 'GET' && /\/api\/clientes(\?|$)/.test(url)) {
        return jsonResponse(asPage([]))
      }
      return jsonResponse('missing', 404)
    })

    await renderWithProviders({ route: '/app/clientes/1' })
    expect(await screen.findAllByText(/cliente a/i)).not.toHaveLength(0)
    expect(screen.getByRole('link', { name: /^editar$/i })).toBeInTheDocument()
  })

  it('deletes cliente after confirmation', async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input)
      const method = init?.method ?? 'GET'
      if (url.includes('/api/me')) {
        return jsonResponse({
          user: { id: 1, email: 'u@test.com' },
          companies: [],
          company_ativa: { id: 1, nome: 'C' },
          prestadores: [],
          prestador_ativo: { id: 1, cpf_cnpj: '1', razao_social: 'E' },
          papel: 'p',
        })
      }
      if (method === 'GET' && /\/api\/clientes(\?|$)/.test(url)) {
        return jsonResponse(asPage([{ id: 1, cpf_cnpj: '123', razao_social: 'Antigo' }]))
      }
      if (method === 'DELETE' && url.includes('/api/clientes/1')) {
        return jsonResponse(undefined, 204)
      }
      return jsonResponse('missing', 404)
    })
    vi.stubGlobal('fetch', fetchMock)

    await renderWithProviders({ route: '/app/clientes' })
    await screen.findByText('Antigo')

    await userEvent.click(screen.getByRole('button', { name: /^excluir$/i }))
    const dialog = await screen.findByRole('alertdialog')
    await userEvent.click(within(dialog).getByRole('button', { name: /^excluir$/i }))

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        expect.stringContaining('/api/clientes/1'),
        expect.objectContaining({ method: 'DELETE' }),
      )
    })
  })

  it('opens certificados tab on prestador show without prefetching certificados list', async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input)
      const method = init?.method ?? 'GET'
      if (url.includes('/api/me')) {
        return jsonResponse({
          user: { id: 1, email: 'u@test.com' },
          companies: [],
          company_ativa: { id: 1, nome: 'C' },
          prestadores: [],
          prestador_ativo: { id: 1, cpf_cnpj: '1', razao_social: 'E' },
          papel: 'p',
        })
      }
      if (method === 'GET' && url.endsWith('/api/prestadores/3')) {
        return jsonResponse({
          id: 3,
          cpf_cnpj: '12345678000199',
          razao_social: 'Prestador Tab Ltda',
          regime_tributario: '1',
          municipio_id: 1,
        })
      }
      if (method === 'GET' && url.includes('/api/prestadores/3/certificados')) {
        return jsonResponse(asPage([]))
      }
      return jsonResponse('missing', 404)
    })
    vi.stubGlobal('fetch', fetchMock)

    await renderWithProviders({ route: '/app/prestadores/3' })
    expect(await screen.findByRole('tab', { name: /dados do prestador/i })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: /certificados digitais/i })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /^editar$/i })).toBeInTheDocument()

    const certCallsBeforeTab = fetchMock.mock.calls.filter(([u]) =>
      String(u).includes('/api/prestadores/3/certificados'),
    )
    expect(certCallsBeforeTab).toHaveLength(0)

    await userEvent.click(screen.getByRole('tab', { name: /certificados digitais/i }))
    expect(await screen.findByRole('button', { name: /enviar certificado/i })).toBeInTheDocument()

    await waitFor(() => {
      expect(
        fetchMock.mock.calls.some(([u]) => String(u).includes('/api/prestadores/3/certificados')),
      ).toBe(true)
    })
  })

  it('opens nested itens on documento show', async () => {
    stubFetch((url, method) => {
      if (method === 'GET' && url.endsWith('/api/documentos/9')) {
        return jsonResponse({
          id: 9,
          modelo: '55',
          serie: '1',
          numero_documento: 1,
          status: 'novo',
          xml_assinado: '<?xml version="1.0"?><NFe/>',
        })
      }
      if (method === 'GET' && url.includes('/api/documentos/9/itens')) {
        return jsonResponse(asPage([{ id: 1, numero_item: 1, descricao: 'Item 1' }]))
      }
      if (method === 'GET' && url.includes('/api/documentos/9/logs-retorno')) {
        return jsonResponse(asPage([]))
      }
      return jsonResponse('missing', 404)
    })

    await renderWithProviders({ route: '/app/documentos/9' })
    await userEvent.click(await screen.findByRole('tab', { name: /itens da nota/i }))
    expect(await screen.findByRole('button', { name: /adicionar item/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /baixar xml/i })).toBeEnabled()
    expect(screen.getByRole('button', { name: /assinar nota fiscal/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /transmitir sefin/i })).toBeInTheDocument()
  })

  it('shows read-only SEFAZ logs on documento show without add form', async () => {
    stubFetch((url, method) => {
      if (method === 'GET' && url.endsWith('/api/documentos/9')) {
        return jsonResponse({
          id: 9,
          modelo: 'NFSe',
          serie: '1',
          numero_documento: 1,
          status: 'rascunho',
        })
      }
      if (method === 'GET' && url.includes('/api/documentos/9/itens')) {
        return jsonResponse(asPage([]))
      }
      if (method === 'GET' && url.includes('/api/documentos/9/logs-retorno')) {
        return jsonResponse(
          asPage([
            {
              id: 1,
              codigo_erro: 'E1',
              mensagem_retorno: 'Falha',
              correcao_sugerida: null,
              data_registro: '2026-01-01T12:00:00Z',
            },
          ]),
        )
      }
      return jsonResponse('missing', 404)
    })

    await renderWithProviders({ route: '/app/documentos/9' })
    await userEvent.click(await screen.findByRole('tab', { name: /logs sefaz/i }))
    const logsTitle = await screen.findByText('Logs de retorno (SEFAZ)')
    await screen.findByText('E1')
    expect(screen.getByRole('columnheader', { name: /mensagem/i })).toBeInTheDocument()
    expect(screen.getByRole('columnheader', { name: /registrado em/i })).toBeInTheDocument()
    expect(screen.getByText('Falha')).toBeInTheDocument()
    expect(screen.getByText(/01\/01\/2026 12:00:00/)).toBeInTheDocument()
    const logsCard = logsTitle.closest('[data-slot="card"]')
    expect(logsCard).toBeTruthy()
    expect(within(logsCard as HTMLElement).queryByRole('button', { name: /adicionar item/i })).toBeNull()
    expect(screen.queryByRole('button', { name: /adicionar item/i })).toBeNull()
  })
})
