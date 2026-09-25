import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  apiFetch,
  authApi,
  createApiError,
  isApiError,
  resourceApi,
  singletonResourceApi,
  tenantApi,
} from './client'

afterEach(() => {
  vi.restoreAllMocks()
})

describe('apiFetch', () => {
  it('returns JSON on success', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        status: 200,
        json: async () => ({ ok: true }),
      }),
    )
    const data = await apiFetch<{ ok: boolean }>('/api/me')
    expect(data.ok).toBe(true)
  })

  it('throws api error on failure', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: false,
        status: 403,
        text: async () => 'forbidden',
      }),
    )
    await expect(apiFetch('/api/x')).rejects.toSatisfy(isApiError)
  })

  it('handles 204 as undefined', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        status: 204,
      }),
    )
    const data = await apiFetch<void>('/tenant/empresa', { method: 'POST' })
    expect(data).toBeUndefined()
  })
})

describe('createApiError', () => {
  it('identifies api errors', () => {
    const err = createApiError(401, 'nope')
    expect(isApiError(err)).toBe(true)
    expect(err.status).toBe(401)
  })
})

describe('resourceApi', () => {
  it('calls list endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => [],
    })
    vi.stubGlobal('fetch', fetchMock)
    await resourceApi('/api/clientes').list()
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/clientes',
      expect.objectContaining({ credentials: 'include' }),
    )
  })

  it('creates and updates records', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({ id: 1 }),
      })
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({ id: 1, name: 'x' }),
      })
    vi.stubGlobal('fetch', fetchMock)
    const api = resourceApi<{ id: number }>('/api/clientes')
    await api.create({ name: 'a' })
    await api.update(1, { name: 'x' })
    expect(fetchMock).toHaveBeenCalledTimes(2)
  })
})

describe('singletonResourceApi', () => {
  it('gets and patches singleton', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({ razao_social: 'A' }),
      })
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({ razao_social: 'B' }),
      })
    vi.stubGlobal('fetch', fetchMock)
    const api = singletonResourceApi('/api/empresa')
    await api.get()
    await api.update({ razao_social: 'B' })
    expect(fetchMock).toHaveBeenCalledTimes(2)
  })
})

describe('auth and tenant helpers', () => {
  it('signs up and signs out', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({ id: 2, email: 'n@t.com' }),
      })
      .mockResolvedValueOnce({
        ok: true,
        status: 204,
      })
    vi.stubGlobal('fetch', fetchMock)
    await authApi.signUp('n@t.com', 'pw')
    await authApi.signOut()
    expect(fetchMock).toHaveBeenCalledTimes(2)
  })

  it('signs in and switches tenant', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({ id: 1, email: 'a@b.com' }),
      })
      .mockResolvedValueOnce({
        ok: true,
        status: 204,
      })
    vi.stubGlobal('fetch', fetchMock)
    await authApi.signIn('a@b.com', 'x')
    await tenantApi.switchEmpresa(10)
    expect(fetchMock).toHaveBeenCalledTimes(2)
  })
})
