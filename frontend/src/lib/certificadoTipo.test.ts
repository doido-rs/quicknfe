import { describe, expect, it } from 'vitest'
import { isCertificadoTipo } from '@/lib/certificadoTipo'

describe('certificadoTipo', () => {
  it('accepts allowed enum values', () => {
    expect(isCertificadoTipo('A1')).toBe(true)
    expect(isCertificadoTipo('PEM')).toBe(true)
    expect(isCertificadoTipo('X')).toBe(false)
  })
})
