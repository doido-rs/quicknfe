import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import * as client from '../api/client'
import { stubApiFetch } from '../test/mockFetch'
import { renderWithProviders } from '../test/renderWithProviders'
import { RegisterPage } from './RegisterPage'

describe('RegisterPage', () => {
  it('submits sign up', async () => {
    stubApiFetch({})
    vi.spyOn(client.authApi, 'signUp').mockResolvedValue({
      id: 2,
      email: 'new@test.com',
    })

    renderWithProviders(<RegisterPage />, { route: '/register' })
    await userEvent.type(screen.getByLabelText(/e-mail/i), 'new@test.com')
    await userEvent.type(screen.getByLabelText(/senha/i), 'secret123')
    await userEvent.click(screen.getByRole('button', { name: /cadastrar/i }))

    await waitFor(() => {
      expect(client.authApi.signUp).toHaveBeenCalledWith('new@test.com', 'secret123')
    })
  })

  it('shows error when sign up fails', async () => {
    stubApiFetch({})
    vi.spyOn(client.authApi, 'signUp').mockRejectedValue(new Error('boom'))

    renderWithProviders(<RegisterPage />, { route: '/register' })
    await userEvent.type(screen.getByLabelText(/e-mail/i), 'x@test.com')
    await userEvent.type(screen.getByLabelText(/senha/i), 'secret123')
    await userEvent.click(screen.getByRole('button', { name: /cadastrar/i }))

    expect(await screen.findByText(/Falha no cadastro/i)).toBeInTheDocument()
  })
})
