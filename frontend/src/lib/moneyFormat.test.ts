import { describe, expect, it } from 'vitest'
import {
  formatMoneyForDisplay,
  moneyFormToNumber,
  normalizeMoneyToForm,
  parseMoneyFromDisplay,
} from './moneyFormat'

describe('moneyFormat', () => {
  it('normalizes API numbers to two decimals', () => {
    expect(normalizeMoneyToForm(100)).toBe('100.00')
    expect(normalizeMoneyToForm(10.5)).toBe('10.50')
  })

  it('parses pt-BR input', () => {
    expect(parseMoneyFromDisplay('1.234,56')).toBe(1234.56)
    expect(parseMoneyFromDisplay('10,5')).toBe(10.5)
  })

  it('formats display with two decimals', () => {
    expect(formatMoneyForDisplay('1234.5')).toBe('1.234,50')
  })

  it('converts form value to API number', () => {
    expect(moneyFormToNumber('99.99')).toBe(99.99)
    expect(moneyFormToNumber('')).toBeNull()
  })
})
