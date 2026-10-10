import type { ResourceDef } from '@/resources/definitions'

export function resourceBasePath(def: ResourceDef) {
  return `/app/${def.key}` as const
}

export function resourceListTo(def: ResourceDef) {
  return {
    to: '/app/$resourceKey' as const,
    params: { resourceKey: def.key },
  }
}

export function resourceNewTo(def: ResourceDef) {
  return {
    to: '/app/$resourceKey/new' as const,
    params: { resourceKey: def.key },
  }
}

export function resourceShowTo(def: ResourceDef, id: number | string) {
  return {
    to: '/app/$resourceKey/$id' as const,
    params: { resourceKey: def.key, id: String(id) },
  }
}

export function resourceEditTo(def: ResourceDef, id: number | string) {
  return {
    to: '/app/$resourceKey/$id/edit' as const,
    params: { resourceKey: def.key, id: String(id) },
  }
}

/** @deprecated Prefer resourceListTo for Link / navigate */
export function resourceListPath(def: ResourceDef) {
  return resourceBasePath(def)
}

export function resourceNewPath(def: ResourceDef) {
  return `${resourceBasePath(def)}/new`
}

export function resourceShowPath(def: ResourceDef, id: number | string) {
  return `${resourceBasePath(def)}/${id}`
}

export function resourceEditPath(def: ResourceDef, id: number | string) {
  return `${resourceBasePath(def)}/${id}/edit`
}

export const defaultResourceListTo = {
  to: '/app/$resourceKey' as const,
  params: { resourceKey: 'documentos' },
}

export type ResourceRouteLink =
  | ReturnType<typeof resourceListTo>
  | ReturnType<typeof resourceNewTo>
  | ReturnType<typeof resourceShowTo>
  | ReturnType<typeof resourceEditTo>
