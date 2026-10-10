import { Navigate, Outlet, useParams } from 'react-router-dom'
import { resources, type ResourceDef } from '@/resources/definitions'

export type ResourceOutletContext = { def: ResourceDef }

export function ResourceLayout() {
  const { resourceKey } = useParams<{ resourceKey: string }>()
  const def = resources.find((r) => r.key === resourceKey)

  if (!def) {
    return <Navigate to="/app/documentos" replace />
  }

  return <Outlet context={{ def } satisfies ResourceOutletContext} />
}
