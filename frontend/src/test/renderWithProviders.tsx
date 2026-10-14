import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { createMemoryHistory, createRouter, RouterProvider } from '@tanstack/react-router'
import { render, type RenderOptions } from '@testing-library/react'
import type { ReactElement } from 'react'
import { AuthProvider } from '../auth/AuthContext'
import { routeTree } from '../router/routeTree'
import { DateFormatProvider } from '../theme/DateFormatContext'
import { ThemeProvider } from '../theme/ThemeContext'

export type RenderWithProvidersOptions = RenderOptions & { route?: string }

function createTestRouter(initialPath: string) {
  return createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [initialPath] }),
  })
}

export async function renderWithProviders(
  uiOrOptions?: ReactElement | RenderWithProvidersOptions,
  maybeOptions?: RenderWithProvidersOptions,
) {
  const usingLegacyUi =
    uiOrOptions != null && typeof uiOrOptions === 'object' && 'type' in uiOrOptions

  const options: RenderWithProvidersOptions = usingLegacyUi
    ? (maybeOptions ?? {})
    : ((uiOrOptions as RenderWithProvidersOptions | undefined) ?? {})

  const route = options.route ?? '/'

  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  const router = createTestRouter(route)
  await router.load()

  const { route: _route, ...renderOptions } = options

  return render(
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        <AuthProvider>
          <DateFormatProvider>
            <RouterProvider router={router} />
          </DateFormatProvider>
        </AuthProvider>
      </ThemeProvider>
    </QueryClientProvider>,
    renderOptions,
  )
}
