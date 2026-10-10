import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { Route, Routes } from 'react-router-dom'
import * as client from '../api/client'
import { stubApiFetch } from '../test/mockFetch'
import { renderWithProviders } from '../test/renderWithProviders'
import { AppLayout } from './AppLayout'

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
    })

    renderWithProviders(
      <Routes>
        <Route path="/app" element={<AppLayout />}>
          <Route index element={<p>Home</p>} />
        </Route>
      </Routes>,
      { route: '/app' },
    )

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

    renderWithProviders(
      <Routes>
        <Route path="/app" element={<AppLayout />}>
          <Route index element={<p>Home</p>} />
        </Route>
        <Route path="/tenant" element={<p>Tenant picker</p>} />
      </Routes>,
      { route: '/app' },
    )

    expect(await screen.findByText(/Tenant picker/i)).toBeInTheDocument()
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
    })
    const signOut = vi.spyOn(client.authApi, 'signOut').mockResolvedValue(undefined)

    renderWithProviders(
      <Routes>
        <Route path="/app" element={<AppLayout />}>
          <Route index element={<p>Home</p>} />
        </Route>
      </Routes>,
      { route: '/app' },
    )

    await screen.findByRole('link', { name: /Notas fiscais/i })
    await userEvent.click(screen.getByRole('button', { name: /^sair$/i }))

    await waitFor(() => {
      expect(signOut).toHaveBeenCalled()
    })
  })
})
