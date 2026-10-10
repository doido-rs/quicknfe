import { vi } from 'vitest'

function jsonResponse(data: unknown, status = 200) {
  return {
    ok: status >= 200 && status < 300,
    status,
    text: async () => '',
    json: async () => data,
  }
}

/** Responde `/api/me` e demais paths usados nos testes de UI. */
export function stubApiFetch(routes: Record<string, unknown>) {
  const handler = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const url = typeof input === 'string' ? input : input.toString()
    const method = init?.method ?? 'GET'

    if (url.includes('/api/me') && method === 'GET') {
      return jsonResponse(
        routes['/api/me'] ?? {
          user: { id: 1, email: 'u@test.com' },
          companies: [],
          company_ativa: { id: 1, nome: 'Company Test' },
          prestadores: [{ id: 1, cpf_cnpj: '1', razao_social: 'Prestador Test' }],
          prestador_ativo: { id: 1, cpf_cnpj: '1', razao_social: 'Prestador Test' },
          papel: 'proprietario',
        },
      )
    }

    for (const [path, body] of Object.entries(routes)) {
      if (path !== '/api/me' && url.includes(path)) {
        return jsonResponse(body)
      }
    }

    return jsonResponse(`unexpected fetch ${method} ${url}`, 404)
  })

  vi.stubGlobal('fetch', handler)
  return handler
}
