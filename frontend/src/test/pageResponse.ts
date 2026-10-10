import type { PageResponse } from '@/api/pagination'

/** Normaliza mocks de listagem para o envelope paginado da API. */
export function asPage<T>(items: T[], nextPageToken: string | null = null): PageResponse<T> {
  return { items, next_page_token: nextPageToken }
}
