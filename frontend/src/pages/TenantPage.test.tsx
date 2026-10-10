import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import * as client from '../api/client'
import { stubApiFetch } from '../test/mockFetch'
import { renderWithProviders } from '../test/renderWithProviders'

afterEach(() => {
  vi.restoreAllMocks()
})

describe('TenantPage', () => {
  it('lists companies when authenticated', async () => {
    vi.spyOn(client, 'apiFetch').mockResolvedValue({
      user: { id: 1, email: 'u@test.com' },
      companies: [
        {
          company: { id: 10, nome: 'Company X' },
          papel: 'proprietario',
        },
      ],
      company_ativa: null,
      prestadores: [],
      prestador_ativo: null,
      papel: null,
    })

    await renderWithProviders({ route: '/tenant' })
    expect(await screen.findByText(/Company X/i)).toBeInTheDocument()
  })

  it('selects company on click', async () => {
    stubApiFetch({
      '/api/me': {
        user: { id: 1, email: 'u@test.com' },
        companies: [
          {
            company: { id: 10, nome: 'Company X' },
            papel: 'proprietario',
          },
        ],
        company_ativa: null,
        prestadores: [],
        prestador_ativo: null,
        papel: null,
      },
    })
    const switchSpy = vi.spyOn(client.tenantApi, 'switchCompany').mockResolvedValue(undefined)

    await renderWithProviders({ route: '/tenant' })
    await screen.findByText(/Company X/i)
    await userEvent.click(screen.getByRole('button', { name: /Company X/i }))

    await waitFor(() => {
      expect(switchSpy).toHaveBeenCalledWith(10)
    })
  })

  it('shows error when selection fails', async () => {
    vi.spyOn(client, 'apiFetch').mockResolvedValue({
      user: { id: 1, email: 'u@test.com' },
      companies: [
        {
          company: { id: 10, nome: 'Company X' },
          papel: 'proprietario',
        },
      ],
      company_ativa: null,
      prestadores: [],
      prestador_ativo: null,
      papel: null,
    })
    vi.spyOn(client.tenantApi, 'switchCompany').mockRejectedValue(
      client.createApiError(403, 'sem acesso'),
    )

    await renderWithProviders({ route: '/tenant' })
    await screen.findByText(/Company X/i)
    await userEvent.click(screen.getByRole('button', { name: /Company X/i }))

    expect(await screen.findByText(/sem acesso/i)).toBeInTheDocument()
  })

  it('signs out from tenant page', async () => {
    vi.spyOn(client, 'apiFetch').mockResolvedValue({
      user: { id: 1, email: 'u@test.com' },
      companies: [],
      company_ativa: null,
      prestadores: [],
      prestador_ativo: null,
      papel: null,
    })
    const signOut = vi.spyOn(client.authApi, 'signOut').mockResolvedValue(undefined)

    await renderWithProviders({ route: '/tenant' })
    await userEvent.click(await screen.findByRole('button', { name: /^sair$/i }))

    await waitFor(() => {
      expect(signOut).toHaveBeenCalled()
    })
  })
})
