import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react'

type PageTitleContextValue = {
  title: string | null
  setTitle: (title: string | null) => void
}

const PageTitleContext = createContext<PageTitleContextValue | null>(null)

export function PageTitleProvider({ children }: { children: ReactNode }) {
  const [title, setTitle] = useState<string | null>(null)
  const value = useMemo(() => ({ title, setTitle }), [title])
  return <PageTitleContext.Provider value={value}>{children}</PageTitleContext.Provider>
}

function usePageTitleContext() {
  const ctx = useContext(PageTitleContext)
  if (!ctx) {
    throw new Error('PageTitle must be used within PageTitleProvider')
  }
  return ctx
}

/** Define o título exibido no topbar (limpa ao desmontar). */
export function PageTitle({ children }: { children: string }) {
  const { setTitle } = usePageTitleContext()
  useEffect(() => {
    setTitle(children)
    return () => setTitle(null)
  }, [children, setTitle])
  return null
}

export function usePageTitle() {
  return usePageTitleContext().title
}
