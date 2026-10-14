import { useBlocker } from '@tanstack/react-router'
import { useCallback, useEffect, useId, useMemo, useRef, useState } from 'react'
import {
  createContext,
  useContext,
  useSyncExternalStore,
  type ReactNode,
} from 'react'

export type UnsavedSaveHandler = () => Promise<boolean>

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

type UnsavedProviderValue = {
  store: UnsavedStore
  saveHandlerRef: React.MutableRefObject<UnsavedSaveHandler | null>
  subscribeSaveHandler: (listener: () => void) => () => void
  getSaveHandlerSnapshot: () => UnsavedSaveHandler | null
  notifySaveHandler: () => void
}

const UnsavedChangesContext = createContext<UnsavedProviderValue | null>(null)

export function UnsavedChangesProvider({ children }: { children: ReactNode }) {
  const store = useMemo(() => createUnsavedStore(), [])
  const saveHandlerRef = useRef<UnsavedSaveHandler | null>(null)
  const saveHandlerListeners = useMemo(() => new Set<() => void>(), [])
  const notifySaveHandlerRef = useRef<() => void>(() => {})
  notifySaveHandlerRef.current = () => {
    for (const l of saveHandlerListeners) l()
  }
  const subscribeSaveHandler = useCallback(
    (listener: () => void) => {
      saveHandlerListeners.add(listener)
      return () => saveHandlerListeners.delete(listener)
    },
    [saveHandlerListeners],
  )
  const getSaveHandlerSnapshot = useCallback(() => saveHandlerRef.current, [])
  const value = useMemo(
    () => ({
      store,
      saveHandlerRef,
      subscribeSaveHandler,
      getSaveHandlerSnapshot,
      notifySaveHandler: () => notifySaveHandlerRef.current(),
    }),
    [store, subscribeSaveHandler, getSaveHandlerSnapshot],
  )

  return (
    <UnsavedChangesContext.Provider value={value}>{children}</UnsavedChangesContext.Provider>
  )
}

function useUnsavedProviderValue() {
  const ctx = useContext(UnsavedChangesContext)
  if (!ctx) {
    throw new Error('useUnsavedFormGuard must be used within UnsavedChangesProvider')
  }
  return ctx
}

function useUnsavedStore() {
  return useUnsavedProviderValue().store
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

/** Registra callback de salvar (ex.: página de edição) para o diálogo de navegação. */
export function useRegisterUnsavedSave(handler: UnsavedSaveHandler | null) {
  const ctx = useUnsavedProviderValue()
  useEffect(() => {
    ctx.saveHandlerRef.current = handler
    ctx.notifySaveHandler()
    return () => {
      if (ctx.saveHandlerRef.current === handler) {
        ctx.saveHandlerRef.current = null
        ctx.notifySaveHandler()
      }
    }
  }, [handler, ctx])
}

export function useOptionalUnsavedSave(): UnsavedSaveHandler | null {
  const { subscribeSaveHandler, getSaveHandlerSnapshot } = useUnsavedProviderValue()
  return useSyncExternalStore(subscribeSaveHandler, getSaveHandlerSnapshot, getSaveHandlerSnapshot)
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

  const confirmSavedAndLeave = useCallback(() => {
    const run = pending
    setPending(null)
    run?.()
  }, [pending])

  return {
    requestNavigation,
    pendingConfirmation: pending != null,
    confirmDiscard,
    cancelNavigation,
    confirmSavedAndLeave,
  }
}
