import { describe, expect, it } from 'vitest'
import {
  documentoListDate,
  documentoListValor,
  documentoParteListLabel,
  documentoStatusLabel,
} from './documentoListFormat'

describe('documentoListFormat', () => {
  it('documentoStatusLabel maps known status and fallback', () => {
    expect(documentoStatusLabel('novo')).toBe('Novo')
    expect(documentoStatusLabel('custom')).toBe('custom')
    expect(documentoStatusLabel(null)).toBe('—')
  })

  it('documentoListDate prefers competencia', () => {
    expect(
      documentoListDate(
        { data_competencia: '2026-10-01', data_emissao: '2026-10-02T12:00:00Z' },
        'DMY',
      ),
    ).toBe('01/10/2026')
  })

  it('documentoListValor formats BRL', () => {
    expect(documentoListValor({ valor_total_servicos_produtos: 1500.5 })).toMatch(/1\.500/)
    expect(documentoListValor({ valor_total_servicos_produtos: 'x' })).toBe('—')
  })

  it('documentoParteListLabel', () => {
    expect(documentoParteListLabel('Tomador')).toBe('Tomador')
    expect(documentoParteListLabel('')).toBe('—')
  })
})
