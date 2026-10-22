import { describe, expect, it } from 'vitest'
import {
  formatPercentForDisplay,
  formatPercentValueForDisplay,
  normalizePercentToForm,
  parsePercentFromDisplay,
  percentFormToNumber,
} from './percentFormat'

describe('percentFormat', () => {
  it('normalizes API numbers to four decimals', () => {
    expect(normalizePercentToForm(5)).toBe('5.0000')
    expect(normalizePercentToForm(2.5)).toBe('2.5000')
    expect(normalizePercentToForm(2.56789)).toBe('2.5679')
  })

  it('parses pt-BR input', () => {
    expect(parsePercentFromDisplay('2,5')).toBe(2.5)
    expect(parsePercentFromDisplay('2,5678%')).toBe(2.5678)
  })

  it('formats display', () => {
    expect(formatPercentForDisplay('5')).toMatch(/5/)
    expect(formatPercentValueForDisplay(2.5)).toBe('2,50%')
    expect(formatPercentValueForDisplay(2.5678)).toBe('2,5678%')
  })

  it('converts form value to API number', () => {
    expect(percentFormToNumber('5.2500')).toBe(5.25)
    expect(percentFormToNumber('')).toBeNull()
  })
})
