import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { apiFetch, authApi, tenantApi } from '../api/client'
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
  const [bootstrapped, setBootstrapped] = useState(false)

  const { data: me, isLoading, refetch } = useQuery({
    queryKey: ['me'],
    queryFn: () => apiFetch<MeResponse>('/api/me'),
    retry: false,
  })

  const refresh = useCallback(async () => {
    await refetch()
    setBootstrapped(true)
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
    queryClient.setQueryData(['me'], undefined)
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
      isLoading: isLoading || !bootstrapped,
      refresh,
      signIn,
      signUp,
      signOut,
      selectEmpresa,
    }),
    [me, isLoading, bootstrapped, refresh, signIn, signUp, signOut, selectEmpresa],
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
