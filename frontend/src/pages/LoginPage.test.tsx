import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import * as client from '../api/client'
import { renderWithProviders } from '../test/renderWithProviders'
import { LoginPage } from './LoginPage'

describe('LoginPage', () => {
  it('submits credentials', async () => {
    vi.spyOn(client.authApi, 'signIn').mockResolvedValue({
      id: 1,
      email: 'a@b.com',
    })
    vi.spyOn(client, 'apiFetch').mockRejectedValue(client.createApiError(401, 'n/a'))

    renderWithProviders(<LoginPage />, { route: '/login' })
    await userEvent.type(screen.getByLabelText(/e-mail/i), 'a@b.com')
    await userEvent.type(screen.getByLabelText(/senha/i), 'secret123')
    await userEvent.click(screen.getByRole('button', { name: /entrar/i }))

    await waitFor(() => {
      expect(client.authApi.signIn).toHaveBeenCalledWith('a@b.com', 'secret123')
    })
  })

  it('shows error on failed login', async () => {
    vi.spyOn(client, 'apiFetch').mockRejectedValue(client.createApiError(401, 'n/a'))
    vi.spyOn(client.authApi, 'signIn').mockRejectedValue(
      client.createApiError(401, 'Credenciais inválidas'),
    )

    renderWithProviders(<LoginPage />, { route: '/login' })
    await userEvent.type(screen.getByLabelText(/e-mail/i), 'a@b.com')
    await userEvent.type(screen.getByLabelText(/senha/i), 'wrong')
    await userEvent.click(screen.getByRole('button', { name: /entrar/i }))

    expect(await screen.findByText(/Credenciais inválidas/i)).toBeInTheDocument()
  })
})
