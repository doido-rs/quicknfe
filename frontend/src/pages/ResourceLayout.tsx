import { Navigate, Outlet, useParams } from '@tanstack/react-router'
import { defaultResourceListTo } from '@/lib/resourcePaths'
import { resources } from '@/resources/definitions'

export function ResourceLayout() {
  const { resourceKey } = useParams({ strict: false })
  const def = resources.find((r) => r.key === resourceKey)

  if (!def) {
    return <Navigate {...defaultResourceListTo} replace />
  }

  return <Outlet />
}
