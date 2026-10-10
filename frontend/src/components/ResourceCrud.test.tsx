import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { resources } from '../resources/definitions'
import { renderWithProviders } from '../test/renderWithProviders'
import { ResourceCrud } from './ResourceCrud'

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

describe('ResourceCrud', () => {
  it('renders clientes list', async () => {
    stubFetch((url, method) => {
      if (method === 'GET' && url.endsWith('/api/clientes')) {
        return jsonResponse([{ id: 1, cpf_cnpj: '123', razao_social: 'Cliente A' }])
      }
      return jsonResponse('missing', 404)
    })

    const def = resources.find((r) => r.key === 'clientes')!
    renderWithProviders(<ResourceCrud def={def} />)

    expect(await screen.findByText('123')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /criar/i })).toBeInTheDocument()
  })

  it('renders prestadores list', async () => {
    stubFetch((url, method) => {
      if (method === 'GET' && url.endsWith('/api/prestadores')) {
        return jsonResponse([
          { id: 1, cpf_cnpj: '11222333000199', razao_social: 'Prestador Seed' },
        ])
      }
      return jsonResponse('missing', 404)
    })

    const def = resources.find((r) => r.key === 'prestadores')!
    renderWithProviders(<ResourceCrud def={def} />)

    expect(await screen.findByText('11222333000199')).toBeInTheDocument()
  })

  it('edits and deletes a cliente', async () => {
    stubFetch((url, method) => {
      if (method === 'GET' && url.endsWith('/api/clientes')) {
        return jsonResponse([{ id: 1, cpf_cnpj: '123', razao_social: 'Antigo' }])
      }
      if (method === 'PATCH' && url.includes('/api/clientes/1')) {
        return jsonResponse({ id: 1, cpf_cnpj: '123', razao_social: 'Novo' })
      }
      if (method === 'DELETE' && url.includes('/api/clientes/1')) {
        return jsonResponse(undefined, 204)
      }
      return jsonResponse('missing', 404)
    })

    const def = resources.find((r) => r.key === 'clientes')!
    renderWithProviders(<ResourceCrud def={def} />)
    await screen.findByText('123')

    await userEvent.click(screen.getByRole('button', { name: /editar/i }))
    const nameInput = screen.getByLabelText(/razão social/i)
    await userEvent.clear(nameInput)
    await userEvent.type(nameInput, 'Novo')
    await userEvent.click(screen.getByRole('button', { name: /^salvar$/i }))

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /criar/i })).toBeInTheDocument()
    })

    await userEvent.click(screen.getByRole('button', { name: /excluir/i }))
  })

  it('opens nested itens for documentos', async () => {
    stubFetch((url, method) => {
      if (method === 'GET' && url.endsWith('/api/documentos')) {
        return jsonResponse([{ id: 9, modelo: '55', serie: '1', numero_documento: 1, status: 'novo' }])
      }
      if (method === 'GET' && url.includes('/api/documentos/9/itens')) {
        return jsonResponse([{ id: 1, numero_item: 1, descricao: 'Item 1' }])
      }
      return jsonResponse('missing', 404)
    })

    const def = resources.find((r) => r.key === 'documentos')!
    renderWithProviders(<ResourceCrud def={def} />)
    await screen.findByText('55')

    await userEvent.click(screen.getByRole('button', { name: /itens/i }))
    expect(await screen.findByRole('button', { name: /adicionar item/i })).toBeInTheDocument()
    await userEvent.click(screen.getByRole('button', { name: /fechar/i }))
  })

  it('creates nested item', async () => {
    let itens: Record<string, unknown>[] = []
    stubFetch((url, method) => {
      if (method === 'GET' && url.endsWith('/api/documentos')) {
        return jsonResponse([{ id: 9, modelo: '55', serie: '1', numero_documento: 1, status: 'novo' }])
      }
      if (method === 'GET' && url.includes('/api/documentos/9/itens')) {
        return jsonResponse(itens)
      }
      if (method === 'POST' && url.includes('/api/documentos/9/itens')) {
        const created = { id: 2, numero_item: 2, descricao: 'Novo item' }
        itens = [created]
        return jsonResponse(created)
      }
      return jsonResponse('missing', 404)
    })

    const def = resources.find((r) => r.key === 'documentos')!
    renderWithProviders(<ResourceCrud def={def} />)
    await screen.findByText('55')
    await userEvent.click(screen.getByRole('button', { name: /itens/i }))
    await userEvent.type(screen.getByLabelText(/nº item/i), '2')
    await userEvent.type(screen.getByLabelText(/descrição/i), 'Novo item')
    await userEvent.click(screen.getByRole('button', { name: /adicionar item/i }))
    expect(await screen.findByText('2')).toBeInTheDocument()
  })

  it('hides create on read-only logs', async () => {
    stubFetch((url, method) => {
      if (method === 'GET' && url.endsWith('/api/logs-retorno')) {
        return jsonResponse([
          { id: 1, codigo_erro: 'E1', mensagem_retorno: 'Falha', documento_fiscal_id: null, lote_id: null },
        ])
      }
      return jsonResponse('missing', 404)
    })

    const def = resources.find((r) => r.key === 'logs-retorno')!
    renderWithProviders(<ResourceCrud def={def} />)
    await screen.findByText('E1')
    expect(screen.queryByRole('button', { name: /excluir/i })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /criar/i })).not.toBeInTheDocument()
  })
})
