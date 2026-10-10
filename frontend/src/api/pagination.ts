export type PageResponse<T> = {
  items: T[]
  next_page_token: string | null
}

export type ListPageOptions = {
  pageToken?: string
  pageSize?: number
  q?: string
}

export function buildListQuery(options?: ListPageOptions): string {
  const sp = new URLSearchParams()
  if (options?.pageToken) sp.set('page_token', options.pageToken)
  if (options?.pageSize != null) sp.set('page_size', String(options.pageSize))
  if (options?.q?.trim()) sp.set('q', options.q.trim())
  const qs = sp.toString()
  return qs ? `?${qs}` : ''
}
