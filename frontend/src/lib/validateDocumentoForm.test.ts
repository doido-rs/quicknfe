import { describe, expect, it } from 'vitest'
import { validateDocumentoFormValues } from './validateDocumentoForm'

const base: Record<string, string> = {
  data_competencia: '2026-10-21',
  valor_total_servicos_produtos: '100',
  tomador_localidade: 'nao_informado',
  intermediario_localidade: 'nao_informado',
  issqn_exigibilidade_suspensa: 'false',
}

describe('validateDocumentoFormValues', () => {
  it('passes minimal valid payload', () => {
    expect(validateDocumentoFormValues(base)).toEqual([])
  })

  it('requires tomador when brasil', () => {
    const errors = validateDocumentoFormValues({
      ...base,
      tomador_localidade: 'brasil',
    })
    expect(errors.some((e) => e.field === 'cliente_id')).toBe(true)
  })

  it('requires processo when issqn suspensa', () => {
    const errors = validateDocumentoFormValues({
      ...base,
      issqn_exigibilidade_suspensa: 'true',
    })
    expect(errors.some((e) => e.field === 'issqn_n_processo_suspensao')).toBe(true)
  })

  it('rejects zero operacao', () => {
    const errors = validateDocumentoFormValues({
      ...base,
      valor_total_servicos_produtos: '0',
    })
    expect(errors.some((e) => e.field === 'valor_total_servicos_produtos')).toBe(true)
  })
})
