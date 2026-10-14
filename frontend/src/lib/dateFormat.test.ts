import { describe, expect, it } from 'vitest'
import {
  formatDateForDisplay,
  formatYmdForDisplay,
  normalizeApiDateToForm,
  parseDateFromDisplay,
} from './dateFormat'

describe('dateFormat', () => {
  it('normalizes ISO to YMD form value', () => {
    expect(normalizeApiDateToForm('2026-10-14T15:00:00Z')).toBe('2026-10-14')
  })

  it('formats DMY MDY YMD', () => {
    expect(formatYmdForDisplay('2026-10-14', 'DMY')).toBe('14/10/2026')
    expect(formatYmdForDisplay('2026-10-14', 'MDY')).toBe('10/14/2026')
    expect(formatYmdForDisplay('2026-10-14', 'YMD')).toBe('2026-10-14')
  })

  it('round-trips display to API YMD', () => {
    expect(parseDateFromDisplay('14/10/2026', 'DMY')).toBe('2026-10-14')
    expect(parseDateFromDisplay('10/14/2026', 'MDY')).toBe('2026-10-14')
    expect(parseDateFromDisplay('2026-10-14', 'YMD')).toBe('2026-10-14')
  })

  it('formatDateForDisplay with datetime keeps time suffix', () => {
    expect(formatDateForDisplay('2026-10-14T08:30:00', 'DMY')).toContain('14/10/2026')
  })
})
