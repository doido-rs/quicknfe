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
    expect(await screen.findByText('123')).toBeInTheDocument()
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
    expect(await screen.findByText(/cliente a/i)).toBeInTheDocument()
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
    await screen.findByText('123')

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
      return jsonResponse('missing', 404)
    })

    await renderWithProviders({ route: '/app/documentos/9' })
    expect(await screen.findByRole('button', { name: /adicionar item/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /baixar xml/i })).toBeEnabled()
  })

  it('hides Novo on read-only logs list', async () => {
    stubFetch((url, method) => {
      if (method === 'GET' && /\/api\/logs-retorno(\?|$)/.test(url)) {
        return jsonResponse(
          asPage([
            {
              id: 1,
              codigo_erro: 'E1',
              mensagem_retorno: 'Falha',
              documento_fiscal_id: null,
              lote_id: null,
            },
          ]),
        )
      }
      return jsonResponse('missing', 404)
    })

    await renderWithProviders({ route: '/app/logs-retorno' })
    await screen.findByText('E1')
    expect(screen.queryByRole('link', { name: /^novo$/i })).not.toBeInTheDocument()
    expect(screen.getByRole('link', { name: /^ver$/i })).toBeInTheDocument()
  })
})
