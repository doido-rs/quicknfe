import { describe, expect, it } from 'vitest'
import { buildDocumentoBody } from './documentoParteLocalidade'
import type { FieldDef } from '@/resources/definitions'

const fields: FieldDef[] = [
  { key: 'cliente_id', label: 'Tomador' },
  { key: 'intermediario_id', label: 'Intermediário' },
  { key: 'tomador_localidade', label: 'Loc tomador' },
  { key: 'intermediario_localidade', label: 'Loc int' },
  { key: 'tomador_exterior_razao_social', label: 'Nome' },
  { key: 'tomador_exterior_logradouro', label: 'Log' },
  { key: 'tomador_exterior_numero', label: 'N' },
  { key: 'tomador_exterior_bairro', label: 'B' },
  { key: 'tomador_exterior_cidade', label: 'C' },
  { key: 'tomador_exterior_codigo_postal', label: 'CEP' },
  { key: 'tomador_exterior_estado_regiao', label: 'UF' },
  { key: 'tomador_exterior_pais', label: 'País' },
]

describe('buildDocumentoBody', () => {
  it('clears tomador when nao informado', () => {
    const body = buildDocumentoBody(fields, {
      tomador_localidade: 'nao_informado',
      intermediario_localidade: 'nao_informado',
      cliente_id: '5',
    })
    expect(body.cliente_id).toBeNull()
    expect(body.tomador_localidade).toBe('nao_informado')
  })

  it('builds tomador exterior payload', () => {
    const body = buildDocumentoBody(fields, {
      tomador_localidade: 'exterior',
      intermediario_localidade: 'nao_informado',
      tomador_exterior_razao_social: 'ACME EU',
      tomador_exterior_logradouro: 'Main St',
      tomador_exterior_numero: '1',
      tomador_exterior_bairro: 'Downtown',
      tomador_exterior_cidade: 'NY',
      tomador_exterior_codigo_postal: '10001',
      tomador_exterior_estado_regiao: 'NY',
      tomador_exterior_pais: 'USA',
    })
    expect(body.cliente_id).toBeNull()
    expect(body.tomador_exterior).toMatchObject({ razao_social: 'ACME EU', pais: 'USA' })
  })

  it('fills API defaults for hidden documento fields', () => {
    const body = buildDocumentoBody(fields, {
      tomador_localidade: 'nao_informado',
      intermediario_localidade: 'nao_informado',
      modelo: 'NFSe',
      ambiente: '2',
      tipo_emissao: '1',
      valor_total_servicos_produtos: '0',
      valor_liquido: '0',
    })
    expect(body.modelo).toBe('NFSe')
    expect(body.ambiente).toBe(2)
    expect(body.tipo_emissao).toBe(1)
    expect(body.valor_liquido).toBe(0)
    expect(typeof body.data_emissao).toBe('string')
    expect(String(body.data_emissao)).toMatch(/^\d{4}-\d{2}-\d{2}T/)
  })
})
