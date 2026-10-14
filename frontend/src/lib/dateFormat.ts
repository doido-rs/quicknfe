export type DateFormatPreference = 'DMY' | 'MDY' | 'YMD'

export const DATE_FORMAT_OPTIONS: { value: DateFormatPreference; label: string }[] = [
  { value: 'DMY', label: 'DD/MM/AAAA (Brasil)' },
  { value: 'MDY', label: 'MM/DD/AAAA (EUA)' },
  { value: 'YMD', label: 'AAAA-MM-DD (ISO)' },
]

export const DEFAULT_DATE_FORMAT: DateFormatPreference = 'DMY'

const STORAGE_PREFIX = 'quicknfe.date_format.user.'

export function dateFormatStorageKey(userId: number | string): string {
  return `${STORAGE_PREFIX}${userId}`
}

export function readStoredDateFormat(userId: number | undefined): DateFormatPreference | null {
  if (userId == null || typeof localStorage === 'undefined') return null
  const raw = localStorage.getItem(dateFormatStorageKey(userId))
  if (raw === 'DMY' || raw === 'MDY' || raw === 'YMD') return raw
  return null
}

export function writeStoredDateFormat(userId: number, pref: DateFormatPreference): void {
  if (typeof localStorage === 'undefined') return
  localStorage.setItem(dateFormatStorageKey(userId), pref)
}

export function parseDateFormat(value: unknown): DateFormatPreference {
  if (value === 'MDY' || value === 'YMD' || value === 'DMY') return value
  return DEFAULT_DATE_FORMAT
}

/** Extract YYYY-MM-DD from API / ISO strings. */
/** Data local de hoje em `YYYY-MM-DD` (form/API). */
export function todayApiDate(): string {
  const d = new Date()
  return `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())}`
}

export function normalizeApiDateToForm(value: unknown): string {
  if (value == null || value === '') return ''
  const s = String(value).trim()
  if (!s) return ''
  if (s.includes('T')) return s.slice(0, 10)
  if (/^\d{4}-\d{2}-\d{2}$/.test(s)) return s
  return s
}

function ymdParts(ymd: string): { y: number; m: number; d: number } | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(ymd)
  if (!m) return null
  const y = Number(m[1])
  const mo = Number(m[2])
  const d = Number(m[3])
  if (!Number.isFinite(y) || !Number.isFinite(mo) || !Number.isFinite(d)) return null
  return { y, m: mo, d }
}

function pad2(n: number): string {
  return String(n).padStart(2, '0')
}

export function formatYmdForDisplay(ymd: string, pref: DateFormatPreference): string {
  const norm = normalizeApiDateToForm(ymd)
  const parts = ymdParts(norm)
  if (!parts) return norm || String(ymd)
  const { y, m, d } = parts
  switch (pref) {
    case 'MDY':
      return `${pad2(m)}/${pad2(d)}/${y}`
    case 'YMD':
      return `${y}-${pad2(m)}-${pad2(d)}`
    case 'DMY':
    default:
      return `${pad2(d)}/${pad2(m)}/${y}`
  }
}

export function formatDateForDisplay(value: unknown, pref: DateFormatPreference): string {
  if (value == null || value === '') return '—'
  const s = String(value).trim()
  if (!s) return '—'
  const datePart = normalizeApiDateToForm(s)
  const display = formatYmdForDisplay(datePart, pref)
  if (s.includes('T') && display !== '—') {
    const time = s.slice(11, 19)
    if (time) return `${display} ${time}`
  }
  return display
}

export function parseDateFromDisplay(text: string, pref: DateFormatPreference): string | null {
  const t = text.trim()
  if (!t) return null
  if (/^\d{4}-\d{2}-\d{2}$/.test(t)) return t

  let m: RegExpExecArray | null
  switch (pref) {
    case 'DMY':
      m = /^(\d{1,2})[/.-](\d{1,2})[/.-](\d{4})$/.exec(t)
      if (!m) return null
      return `${m[3]}-${pad2(Number(m[2]))}-${pad2(Number(m[1]))}`
    case 'MDY':
      m = /^(\d{1,2})[/.-](\d{1,2})[/.-](\d{4})$/.exec(t)
      if (!m) return null
      return `${m[3]}-${pad2(Number(m[1]))}-${pad2(Number(m[2]))}`
    case 'YMD':
      m = /^(\d{4})[/.-](\d{1,2})[/.-](\d{1,2})$/.exec(t)
      if (!m) return null
      return `${m[1]}-${pad2(Number(m[2]))}-${pad2(Number(m[3]))}`
    default:
      return null
  }
}

export function formDateToDisplay(formYmd: string, pref: DateFormatPreference): string {
  if (!formYmd.trim()) return ''
  return formatYmdForDisplay(formYmd, pref)
}
