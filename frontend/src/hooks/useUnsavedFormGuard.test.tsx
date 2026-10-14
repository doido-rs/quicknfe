import { describe, expect, it } from 'vitest'
import { renderHook } from '@testing-library/react'
import type { ReactNode } from 'react'
import {
  UnsavedChangesProvider,
  useGlobalUnsavedDirty,
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
})
