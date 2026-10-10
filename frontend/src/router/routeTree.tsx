import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  redirect,
} from '@tanstack/react-router'
import { AppLayout } from '@/pages/AppLayout'
import { DashboardPage } from '@/pages/DashboardPage'
import { LoginPage } from '@/pages/LoginPage'
import { PrestadorPage } from '@/pages/PrestadorPage'
import { RegisterPage } from '@/pages/RegisterPage'
import { ResourceLayout } from '@/pages/ResourceLayout'
import { TenantPage } from '@/pages/TenantPage'
import { ResourceEditPage } from '@/pages/resources/ResourceEditPage'
import { ResourceListPage } from '@/pages/resources/ResourceListPage'
import { ResourceNewPage } from '@/pages/resources/ResourceNewPage'
import { ResourceShowPage } from '@/pages/resources/ResourceShowPage'
import { appDashboardTo } from '@/lib/resourcePaths'

const rootRoute = createRootRoute({
  component: () => <Outlet />,
})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  beforeLoad: () => {
    throw redirect({ to: '/login' })
  },
})

const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/login',
  component: LoginPage,
})

const registerRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/register',
  component: RegisterPage,
})

const tenantRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tenant',
  component: TenantPage,
})

const tenantPrestadorRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tenant/prestador',
  component: PrestadorPage,
})

const appRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/app',
  component: AppLayout,
})

const appIndexRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/',
  beforeLoad: () => {
    throw redirect(appDashboardTo)
  },
})

const dashboardRoute = createRoute({
  getParentRoute: () => appRoute,
  path: 'dashboard',
  component: DashboardPage,
})

const resourceLayoutRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '$resourceKey',
  component: ResourceLayout,
})

const resourceListRoute = createRoute({
  getParentRoute: () => resourceLayoutRoute,
  path: '/',
  component: ResourceListPage,
})

const resourceNewRoute = createRoute({
  getParentRoute: () => resourceLayoutRoute,
  path: 'new',
  component: ResourceNewPage,
})

const resourceShowRoute = createRoute({
  getParentRoute: () => resourceLayoutRoute,
  path: '$id',
  component: ResourceShowPage,
})

const resourceEditRoute = createRoute({
  getParentRoute: () => resourceLayoutRoute,
  path: '$id/edit',
  component: ResourceEditPage,
})

export const routeTree = rootRoute.addChildren([
  indexRoute,
  loginRoute,
  registerRoute,
  tenantRoute,
  tenantPrestadorRoute,
  appRoute.addChildren([
    appIndexRoute,
    dashboardRoute,
    resourceLayoutRoute.addChildren([
      resourceListRoute,
      resourceNewRoute,
      resourceShowRoute,
      resourceEditRoute,
    ]),
  ]),
])

export function createAppRouter() {
  return createRouter({
    routeTree,
    defaultPreload: 'intent',
  })
}

export const router = createAppRouter()

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
