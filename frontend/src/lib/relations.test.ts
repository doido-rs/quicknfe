import { describe, expect, it } from 'vitest'
import { relationApiPath, relationOptionLabel, relationResourceKey } from './relations'

describe('relations', () => {
  it('relationResourceKey from field def or fk map', () => {
    expect(relationResourceKey({ key: 'cliente_id', type: 'relation', relationResource: 'clientes' })).toBe(
      'clientes',
    )
    expect(relationResourceKey({ key: 'cliente_id', type: 'text' })).toBe('clientes')
    expect(relationResourceKey({ key: 'unknown', type: 'text' })).toBeNull()
  })

  it('relationApiPath resolves resource path', () => {
    expect(relationApiPath('clientes')).toBe('/api/clientes')
  })

  it('relationOptionLabel formats cnas and generic resources', () => {
    expect(relationOptionLabel('cnas', { id: 1, codigo: '01.01', descricao: 'Serviço' })).toBe(
      '01.01 — Serviço',
    )
    expect(relationOptionLabel('clientes', { id: 5, razao_social: 'ACME' })).toContain('ACME')
  })
})
