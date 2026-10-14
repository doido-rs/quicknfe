import { act, renderHook, waitFor } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import type { ReactNode } from 'react'
import {
  UnsavedChangesProvider,
  useGlobalUnsavedDirty,
  useGuardedNavigate,
  useUnsavedFormGuard,
} from '@/hooks/useUnsavedFormGuard'

function wrapper({ children }: { children: ReactNode }) {
  return <UnsavedChangesProvider>{children}</UnsavedChangesProvider>
}

describe('useUnsavedFormGuard', () => {
  it('reflects dirty state in global store', () => {
    const { result, rerender } = renderHook(
      ({ dirty }: { dirty: boolean }) => {
        useUnsavedFormGuard(dirty)
        return useGlobalUnsavedDirty()
      },
      { wrapper, initialProps: { dirty: true } },
    )
    expect(result.current).toBe(true)
    rerender({ dirty: false })
    expect(result.current).toBe(false)
  })

  it('queues navigation when dirty via useGuardedNavigate', async () => {
    const { result } = renderHook(
      () => {
        useUnsavedFormGuard(true)
        const dirty = useGlobalUnsavedDirty()
        const nav = useGuardedNavigate()
        return { dirty, nav }
      },
      { wrapper },
    )
    await waitFor(() => expect(result.current.dirty).toBe(true))
    let ran = false
    act(() => {
      result.current.nav.requestNavigation(() => {
        ran = true
      })
    })
    expect(result.current.nav.pendingConfirmation).toBe(true)
    expect(ran).toBe(false)
    act(() => {
      result.current.nav.confirmDiscard()
    })
    expect(ran).toBe(true)
  })
})
