import { describe, expect, it } from 'vitest'
import {
  fieldValue,
  formatFieldValue,
  formatMunicipioRelationLabel,
  rowToForm,
} from './resourceForm'

describe('resourceForm', () => {
  it('maps row to form strings', () => {
    const form = rowToForm({ id: 1, nome: 'X', count: 2 }, [
      { key: 'nome', label: 'Nome' },
      { key: 'count', label: 'Count', type: 'number' },
    ])
    expect(form).toEqual({ nome: 'X', count: '2' })
  })

  it('reads nested field paths', () => {
    expect(fieldValue({ municipio: { uf: 'SP' } }, 'municipio.uf')).toBe('SP')
  })

  it('formats municipio relation from nested row', () => {
    const row = { municipio: { nome: 'São Paulo', uf: 'SP', codigo_ibge: '3550308' } }
    expect(formatMunicipioRelationLabel(row)).toBe('São Paulo (SP · IBGE 3550308)')
    expect(
      formatFieldValue(1, { key: 'municipio_id', relationResource: 'municipios' }, row),
    ).toBe('São Paulo (SP · IBGE 3550308)')
  })

  it('formats empty values', () => {
    expect(formatFieldValue(null)).toBe('—')
    expect(formatFieldValue('ok')).toBe('ok')
  })
})
