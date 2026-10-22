import { describe, expect, it } from 'vitest'
import {
  regEspTribIssqnToFormValue,
  regimeEspecialTribIssqnLabel,
  tribIssqnToFormValue,
  tributacaoIssqnLabel,
} from './documentoTributacaoIssqn'

describe('documentoTributacaoIssqn wire → form', () => {
  it('maps trib_issqn API enum to select value', () => {
    expect(tribIssqnToFormValue('OPERACAO_TRIBUTAVEL')).toBe('1')
    expect(tribIssqnToFormValue(1)).toBe('1')
    expect(tributacaoIssqnLabel('NAO_INCIDENCIA')).toBe('Não incidência')
  })

  it('maps reg_esp_trib_issqn API enum to select value', () => {
    expect(regEspTribIssqnToFormValue('NENHUM')).toBe('0')
    expect(regEspTribIssqnToFormValue('MEI')).toBe('5')
    expect(regimeEspecialTribIssqnLabel('SOCIEDADE_PROFISSIONAIS')).toBe(
      'Sociedade de profissionais',
    )
  })
})
