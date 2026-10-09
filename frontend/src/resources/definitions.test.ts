import { describe, expect, it } from 'vitest'
import { resources } from './definitions'

describe('resources', () => {
  it('includes core tenant resources', () => {
    const keys = resources.map((r) => r.key)
    expect(keys).toContain('empresa')
    expect(keys).toContain('clientes')
    expect(keys).toContain('documentos')
    expect(keys).toContain('intermediarios')
  })

  it('lists notas fiscais first in the menu', () => {
    expect(resources[0]?.key).toBe('documentos')
    expect(resources[0]?.title).toBe('Notas fiscais')
  })

  it('marks logs as read-only create', () => {
    const logs = resources.find((r) => r.key === 'logs-retorno')
    expect(logs?.allowCreate).toBe(false)
  })

  it('builds nested itens path from documento id', () => {
    const documentos = resources.find((r) => r.key === 'documentos')
    expect(documentos?.nested?.path(42)).toBe('/api/documentos/42/itens')
  })
})
