import { describe, expect, it } from 'vitest'
import { buildDocumentoRelationLabels, readOnlyFieldDisplay } from './documentoShowLabels'

describe('documentoShowLabels', () => {
  it('buildDocumentoRelationLabels maps API presentation fields', () => {
    const labels = buildDocumentoRelationLabels({
      prestador_razao_social: 'Prestador SA',
      tomador_razao_social: 'Tomador LTDA',
      municipio_emissao: { nome: 'Rio de Janeiro', uf: 'RJ', codigo_ibge: '3304557' },
    })
    expect(labels.empresa_id).toBe('Prestador SA')
    expect(labels.cliente_id).toBe('Tomador LTDA')
    expect(labels.municipio_emissao_id).toContain('Rio de Janeiro')
  })

  it('readOnlyFieldDisplay prefers relation label', () => {
    const text = readOnlyFieldDisplay(
      { key: 'cliente_id', label: 'Tomador', type: 'relation' },
      '',
      {},
      { cliente_id: 'Tomador X' },
    )
    expect(text).toBe('Tomador X')
  })
})
