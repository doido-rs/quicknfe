import { Navigate, useParams } from 'react-router-dom'
import { ResourceCrud } from '../components/ResourceCrud'
import { resources } from '../resources/definitions'

export function ResourcePage() {
  const { resourceKey } = useParams<{ resourceKey: string }>()
  const def = resources.find((r) => r.key === resourceKey)

  if (!def) {
    return <Navigate to="/app/documentos" replace />
  }

  return <ResourceCrud def={def} />
}
