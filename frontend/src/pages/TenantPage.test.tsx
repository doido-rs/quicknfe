import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import * as client from '../api/client'
import { stubApiFetch } from '../test/mockFetch'
import { renderWithProviders } from '../test/renderWithProviders'
import { TenantPage } from './TenantPage'

afterEach(() => {
  vi.restoreAllMocks()
})

describe('TenantPage', () => {
  it('lists empresas when authenticated', async () => {
    vi.spyOn(client, 'apiFetch').mockResolvedValue({
      user: { id: 1, email: 'u@test.com' },
      empresas: [
        {
          empresa: { id: 10, cpf_cnpj: '1', razao_social: 'Empresa X' },
          papel: 'proprietario',
        },
      ],
      empresa_ativa: null,
      papel: null,
    })

    renderWithProviders(<TenantPage />, { route: '/tenant' })
    expect(await screen.findByText(/Empresa X/i)).toBeInTheDocument()
  })

  it('selects empresa on click', async () => {
    stubApiFetch({
      '/api/me': {
        user: { id: 1, email: 'u@test.com' },
        empresas: [
          {
            empresa: { id: 10, cpf_cnpj: '1', razao_social: 'Empresa X' },
            papel: 'proprietario',
          },
        ],
        empresa_ativa: null,
        papel: null,
      },
    })
    const switchSpy = vi.spyOn(client.tenantApi, 'switchEmpresa').mockResolvedValue(undefined)

    renderWithProviders(<TenantPage />, { route: '/tenant' })
    await screen.findByText(/Empresa X/i)
    await userEvent.click(screen.getByRole('button', { name: /Empresa X/i }))

    await waitFor(() => {
      expect(switchSpy).toHaveBeenCalledWith(10)
    })
  })

  it('shows error when selection fails', async () => {
    vi.spyOn(client, 'apiFetch').mockResolvedValue({
      user: { id: 1, email: 'u@test.com' },
      empresas: [
        {
          empresa: { id: 10, cpf_cnpj: '1', razao_social: 'Empresa X' },
          papel: 'proprietario',
        },
      ],
      empresa_ativa: null,
      papel: null,
    })
    vi.spyOn(client.tenantApi, 'switchEmpresa').mockRejectedValue(
      client.createApiError(403, 'sem acesso'),
    )

    renderWithProviders(<TenantPage />, { route: '/tenant' })
    await screen.findByText(/Empresa X/i)
    await userEvent.click(screen.getByRole('button', { name: /Empresa X/i }))

    expect(await screen.findByText(/sem acesso/i)).toBeInTheDocument()
  })

  it('signs out from tenant page', async () => {
    vi.spyOn(client, 'apiFetch').mockResolvedValue({
      user: { id: 1, email: 'u@test.com' },
      empresas: [],
      empresa_ativa: null,
      papel: null,
    })
    const signOut = vi.spyOn(client.authApi, 'signOut').mockResolvedValue(undefined)

    renderWithProviders(<TenantPage />, { route: '/tenant' })
    await userEvent.click(await screen.findByRole('button', { name: /^sair$/i }))

    await waitFor(() => {
      expect(signOut).toHaveBeenCalled()
    })
  })
})
