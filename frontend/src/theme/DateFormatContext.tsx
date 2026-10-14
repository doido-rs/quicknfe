import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react'
import { useAuth } from '@/auth/AuthContext'
import { apiFetch } from '@/api/client'
import {
  DEFAULT_DATE_FORMAT,
  type DateFormatPreference,
  parseDateFormat,
  readStoredDateFormat,
  writeStoredDateFormat,
} from '@/lib/dateFormat'

interface DateFormatContextValue {
  preference: DateFormatPreference
  setPreference: (pref: DateFormatPreference) => void
}

const DateFormatContext = createContext<DateFormatContextValue | null>(null)

export function DateFormatProvider({ children }: { children: ReactNode }) {
  const { me } = useAuth()
  const userId = me?.user.id
  const fromApi = me?.user.date_format

  const [preference, setPreferenceState] = useState<DateFormatPreference>(() => {
    if (fromApi) return parseDateFormat(fromApi)
    const stored = readStoredDateFormat(userId)
    return stored ?? DEFAULT_DATE_FORMAT
  })

  useEffect(() => {
    if (fromApi) {
      setPreferenceState(parseDateFormat(fromApi))
      return
    }
    const stored = readStoredDateFormat(userId)
    setPreferenceState(stored ?? DEFAULT_DATE_FORMAT)
  }, [fromApi, userId])

  const setPreference = useCallback(
    (pref: DateFormatPreference) => {
      setPreferenceState(pref)
      if (userId != null) {
        writeStoredDateFormat(userId, pref)
        void apiFetch<{ date_format?: string }>('/api/me', {
          method: 'PATCH',
          body: JSON.stringify({ date_format: pref }),
        }).catch(() => {
          /* localStorage already updated */
        })
      }
    },
    [userId],
  )

  const value = useMemo(() => ({ preference, setPreference }), [preference, setPreference])

  return <DateFormatContext.Provider value={value}>{children}</DateFormatContext.Provider>
}

export function useDateFormat(): DateFormatContextValue {
  const ctx = useContext(DateFormatContext)
  if (!ctx) {
    return {
      preference: DEFAULT_DATE_FORMAT,
      setPreference: () => {},
    }
  }
  return ctx
}
