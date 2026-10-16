import { describe, expect, it } from 'vitest'
import { formatBrl } from './format'

describe('formatBrl', () => {
  it('formats currency in pt-BR', () => {
    expect(formatBrl(1500.5)).toMatch(/1\.500/)
  })
})
