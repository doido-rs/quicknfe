import { useParams } from '@tanstack/react-router'
import { resources } from '@/resources/definitions'

export function useResourceDef() {
  const { resourceKey } = useParams({ strict: false })
  const def = resources.find((r) => r.key === resourceKey)
  if (!def) {
    throw new Error(`Unknown resource: ${resourceKey}`)
  }
  return def
}

export function useResourceRecordId() {
  const { id } = useParams({ strict: false })
  const recordId = Number(id)
  return Number.isFinite(recordId) ? recordId : null
}
