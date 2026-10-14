import { useBlocker } from '@tanstack/react-router'
import { useCallback, useEffect, useId, useMemo, useState } from 'react'
import {
  createContext,
  useContext,
  useSyncExternalStore,
  type ReactNode,
} from 'react'

type UnsavedStore = {
  subscribe: (listener: () => void) => () => void
  getSnapshot: () => boolean
  setDirty: (id: string, dirty: boolean) => void
}

function createUnsavedStore(): UnsavedStore {
  const entries = new Map<string, boolean>()
  const listeners = new Set<() => void>()

  const notify = () => {
    for (const l of listeners) l()
  }

  return {
    subscribe: (listener) => {
      listeners.add(listener)
      return () => listeners.delete(listener)
    },
    getSnapshot: () => {
      for (const dirty of entries.values()) {
        if (dirty) return true
      }
      return false
    },
    setDirty: (id, dirty) => {
      if (dirty) {
        entries.set(id, true)
      } else {
        entries.delete(id)
      }
      notify()
    },
  }
}

const UnsavedChangesContext = createContext<UnsavedStore | null>(null)

export function UnsavedChangesProvider({ children }: { children: ReactNode }) {
  const store = useMemo(() => createUnsavedStore(), [])
  return (
    <UnsavedChangesContext.Provider value={store}>{children}</UnsavedChangesContext.Provider>
  )
}

function useUnsavedStore() {
  const ctx = useContext(UnsavedChangesContext)
  if (!ctx) {
    throw new Error('useUnsavedFormGuard must be used within UnsavedChangesProvider')
  }
  return ctx
}

export function useGlobalUnsavedDirty() {
  const store = useUnsavedStore()
  return useSyncExternalStore(store.subscribe, store.getSnapshot, store.getSnapshot)
}

/** Registra dirty state de um formulário (RHF isDirty ou painel aninhado). */
export function useUnsavedFormGuard(isDirty: boolean) {
  const id = useId()
  const store = useUnsavedStore()

  useEffect(() => {
    store.setDirty(id, isDirty)
    return () => store.setDirty(id, false)
  }, [id, isDirty, store])
}

export type UnsavedBlockerState =
  | { status: 'idle' }
  | {
      status: 'blocked'
      proceed: () => void
      reset: () => void
    }

export function useUnsavedNavigationBlocker(): UnsavedBlockerState {
  const isDirty = useGlobalUnsavedDirty()
  const blocker = useBlocker({
    shouldBlockFn: () => isDirty,
    enableBeforeUnload: () => isDirty,
    withResolver: true,
  })

  if (blocker.status === 'blocked') {
    return {
      status: 'blocked',
      proceed: () => blocker.proceed?.(),
      reset: () => blocker.reset?.(),
    }
  }
  return { status: 'idle' }
}

export function useGuardedNavigate() {
  const isDirty = useGlobalUnsavedDirty()
  const [pending, setPending] = useState<(() => void) | null>(null)

  const requestNavigation = useCallback(
    (action: () => void) => {
      if (!isDirty) {
        action()
        return
      }
      setPending(() => action)
    },
    [isDirty],
  )

  const confirmDiscard = useCallback(() => {
    const run = pending
    setPending(null)
    run?.()
  }, [pending])

  const cancelNavigation = useCallback(() => setPending(null), [])

  return {
    requestNavigation,
    pendingConfirmation: pending != null,
    confirmDiscard,
    cancelNavigation,
  }
}
