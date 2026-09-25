import { describe, expect, it } from 'vitest'
import { buildBody, emptyForm, rowLabel } from './resourceForm'

const fields = [
  { key: 'name', label: 'Name', required: true },
  { key: 'count', label: 'Count', type: 'number' as const },
]

describe('resourceForm', () => {
  it('builds empty form', () => {
    expect(emptyForm(fields)).toEqual({ name: '', count: '' })
  })

  it('builds body with numbers and nulls', () => {
    const body = buildBody(fields, { name: 'Acme', count: '3', extra: '' })
    expect(body).toEqual({ name: 'Acme', count: 3 })
  })

  it('labels row from first field', () => {
    expect(rowLabel({ id: 1, name: 'Foo' }, fields)).toBe('Foo')
    expect(rowLabel({ id: 2 }, fields)).toBe('2')
  })
})
