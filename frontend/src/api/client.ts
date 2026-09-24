const API_BASE = import.meta.env.VITE_API_URL ?? ''

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

async function parseError(res: Response): Promise<string> {
  const text = await res.text()
  return text || res.statusText
}

export async function apiFetch<T>(
  path: string,
  init: RequestInit = {},
): Promise<T> {
  const headers = new Headers(init.headers)
  if (!headers.has('Accept')) {
    headers.set('Accept', 'application/json')
  }
  if (init.body && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json')
  }

  const res = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers,
    credentials: 'include',
  })

  if (res.status === 204) {
    return undefined as T
  }

  if (!res.ok) {
    throw new ApiError(res.status, await parseError(res))
  }

  return (await res.json()) as T
}

export const authApi = {
  signIn: (email: string, password: string) =>
    apiFetch<{ id: number; email: string }>('/users/sign_in', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    }),
  signUp: (email: string, password: string) =>
    apiFetch<{ id: number; email: string }>('/users/sign_up', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    }),
  signOut: () =>
    apiFetch<void>('/users/sign_out', { method: 'DELETE' }),
}

export const tenantApi = {
  switchEmpresa: (empresa_id: number) =>
    apiFetch<void>('/tenant/empresa', {
      method: 'POST',
      body: JSON.stringify({ empresa_id }),
    }),
}

export function resourceApi<T>(basePath: string) {
  return {
    list: () => apiFetch<T[]>(basePath),
    get: (id: number) => apiFetch<T>(`${basePath}/${id}`),
    create: (body: unknown) =>
      apiFetch<T>(basePath, { method: 'POST', body: JSON.stringify(body) }),
    update: (id: number, body: unknown) =>
      apiFetch<T>(`${basePath}/${id}`, {
        method: 'PATCH',
        body: JSON.stringify(body),
      }),
    remove: (id: number) =>
      apiFetch<void>(`${basePath}/${id}`, { method: 'DELETE' }),
  }
}
