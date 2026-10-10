import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import * as client from '../api/client'
import { stubApiFetch } from '../test/mockFetch'
import { renderWithProviders } from '../test/renderWithProviders'

afterEach(() => {
  vi.restoreAllMocks()
})

describe('AppLayout', () => {
  it('shows nav when tenant is active', async () => {
    stubApiFetch({
      '/api/me': {
        user: { id: 1, email: 'u@test.com' },
        companies: [],
        company_ativa: { id: 1, nome: 'Company' },
        prestadores: [],
        prestador_ativo: { id: 1, cpf_cnpj: '1', razao_social: 'Minha Empresa' },
        papel: 'proprietario',
      },
      '/api/documentos': { items: [], next_page_token: null },
    })

    await renderWithProviders({ route: '/app/documentos' })

    expect(await screen.findByRole('link', { name: /Notas fiscais/i })).toBeInTheDocument()
  })

  it('redirects to tenant when no empresa ativa', async () => {
    stubApiFetch({
      '/api/me': {
        user: { id: 1, email: 'u@test.com' },
        companies: [{ company: { id: 1, nome: 'C' }, papel: 'p' }],
        company_ativa: null,
        prestadores: [],
        prestador_ativo: null,
        papel: null,
      },
    })

    await renderWithProviders({ route: '/app' })

    expect(await screen.findByText(/Selecione a company/i)).toBeInTheDocument()
  })

  it('calls signOut from header', async () => {
    stubApiFetch({
      '/api/me': {
        user: { id: 1, email: 'u@test.com' },
        companies: [],
        company_ativa: { id: 1, nome: 'Company' },
        prestadores: [],
        prestador_ativo: { id: 1, cpf_cnpj: '1', razao_social: 'Minha Empresa' },
        papel: 'proprietario',
      },
      '/api/documentos': { items: [], next_page_token: null },
    })
    const signOut = vi.spyOn(client.authApi, 'signOut').mockResolvedValue(undefined)

    await renderWithProviders({ route: '/app/documentos' })

    await screen.findByRole('link', { name: /Notas fiscais/i })
    await userEvent.click(screen.getByRole('button', { name: /^sair$/i }))

    await waitFor(() => {
      expect(signOut).toHaveBeenCalled()
    })
  })
})
