import { describe, expect, it } from 'vitest'
import { rowToForm, formatFieldValue } from './resourceForm'

describe('resourceForm', () => {
  it('maps row to form strings', () => {
    const form = rowToForm({ id: 1, nome: 'X', count: 2 }, [
      { key: 'nome', label: 'Nome' },
      { key: 'count', label: 'Count', type: 'number' },
    ])
    expect(form).toEqual({ nome: 'X', count: '2' })
  })

  it('formats empty values', () => {
    expect(formatFieldValue(null)).toBe('—')
    expect(formatFieldValue('ok')).toBe('ok')
  })
})
