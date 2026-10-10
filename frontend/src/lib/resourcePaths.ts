import type { ResourceDef } from '@/resources/definitions'

export function resourceBasePath(def: ResourceDef) {
  return `/app/${def.key}`
}

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
