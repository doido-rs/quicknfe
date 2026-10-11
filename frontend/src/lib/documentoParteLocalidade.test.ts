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
})
