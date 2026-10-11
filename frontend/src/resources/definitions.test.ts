import { describe, expect, it } from 'vitest'
import { resources } from './definitions'

describe('resources', () => {
  it('includes core tenant resources', () => {
    const keys = resources.map((r) => r.key)
    expect(keys).toContain('prestadores')
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

  it('nests certificados under prestador show page', () => {
    const prestadores = resources.find((r) => r.key === 'prestadores')
    expect(prestadores?.nested?.path(7)).toBe('/api/prestadores/7/certificados')
    expect(resources.some((r) => r.key === 'certificados')).toBe(false)
  })

  it('groups nota fiscal form: prestador|partes row then extras', () => {
    const documentos = resources.find((r) => r.key === 'documentos')
    expect(documentos?.formSections).toHaveLength(2)
    expect(documentos?.formSections?.[0]).toMatchObject({
      type: 'prestador-partes-row',
      prestadorFieldKey: 'empresa_id',
      partesFieldKeys: ['cliente_id', 'intermediario_id'],
    })
    expect(documentos?.fields.some((f) => f.key === 'empresa_id')).toBe(true)
    expect(documentos?.formSections?.[1]).toMatchObject({
      type: 'fields',
      title: 'Dados extras',
      columns: 1,
    })
  })
})
