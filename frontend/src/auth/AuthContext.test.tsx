import { act, renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import * as client from '../api/client'
import { stubApiFetch } from '../test/mockFetch'
import { AuthProvider, useAuth } from './AuthContext'

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <AuthProvider>{children}</AuthProvider>
      </QueryClientProvider>
    )
  }
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('AuthContext', () => {
  it('loads session from /api/me', async () => {
    stubApiFetch({
      '/api/me': {
        user: { id: 1, email: 'admin@test.com' },
        empresas: [],
        empresa_ativa: null,
        papel: null,
      },
    })

    const { result } = renderHook(() => useAuth(), { wrapper: createWrapper() })

    await waitFor(() => {
      expect(result.current.me?.user.email).toBe('admin@test.com')
    })
  })

  it('signOut calls API and drops cached session', async () => {
    stubApiFetch({
      '/api/me': {
        user: { id: 1, email: 'admin@test.com' },
        empresas: [],
        empresa_ativa: null,
        papel: null,
      },
    })
    const signOut = vi.spyOn(client.authApi, 'signOut').mockResolvedValue(undefined)

    const { result } = renderHook(() => useAuth(), { wrapper: createWrapper() })

    await waitFor(() => {
      expect(result.current.me?.user.email).toBe('admin@test.com')
    })

    await act(async () => {
      await result.current.signOut()
    })

    expect(signOut).toHaveBeenCalled()
  })
})
