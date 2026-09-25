import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  type ReactNode,
} from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { apiFetch, authApi, isApiError, tenantApi } from '../api/client'
import type { MeResponse } from '../api/types'

interface AuthContextValue {
  me: MeResponse | undefined
  isLoading: boolean
  refresh: () => Promise<void>
  signIn: (email: string, password: string) => Promise<void>
  signUp: (email: string, password: string) => Promise<void>
  signOut: () => Promise<void>
  selectEmpresa: (empresaId: number) => Promise<void>
}

const AuthContext = createContext<AuthContextValue | null>(null)

export function AuthProvider({ children }: { children: ReactNode }) {
  const queryClient = useQueryClient()

  const { data: me, isLoading, refetch } = useQuery({
    queryKey: ['me'],
    queryFn: async () => {
      try {
        return await apiFetch<MeResponse>('/api/me')
      } catch (err) {
        if (isApiError(err) && err.status === 401) {
          return undefined
        }
        throw err
      }
    },
    retry: false,
  })

  const refresh = useCallback(async () => {
    await refetch()
  }, [refetch])

  const signIn = useCallback(
    async (email: string, password: string) => {
      await authApi.signIn(email, password)
      await queryClient.invalidateQueries({ queryKey: ['me'] })
    },
    [queryClient],
  )

  const signUp = useCallback(
    async (email: string, password: string) => {
      await authApi.signUp(email, password)
      await queryClient.invalidateQueries({ queryKey: ['me'] })
    },
    [queryClient],
  )

  const signOut = useCallback(async () => {
    await authApi.signOut()
    queryClient.removeQueries({ queryKey: ['me'] })
  }, [queryClient])

  const selectEmpresa = useCallback(
    async (empresaId: number) => {
      await tenantApi.switchEmpresa(empresaId)
      await queryClient.invalidateQueries({ queryKey: ['me'] })
    },
    [queryClient],
  )

  const value = useMemo(
    () => ({
      me,
      isLoading,
      refresh,
      signIn,
      signUp,
      signOut,
      selectEmpresa,
    }),
    [me, isLoading, refresh, signIn, signUp, signOut, selectEmpresa],
  )

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

export function useAuth() {
  const ctx = useContext(AuthContext)
  if (!ctx) {
    throw new Error('useAuth must be used within AuthProvider')
  }
  return ctx
}
