import { describe, expect, it } from 'vitest'
import {
  cstPisCofinsLabel,
  tpRetPisCofinsCsllLabel,
} from './documentoTributacaoFederal'
import {
  regimeEspecialTribIssqnLabel,
  simNaoLabel,
  tributacaoIssqnLabel,
} from './documentoTributacaoIssqn'
import { regimeTributarioLabel } from './regimeTributario'

describe('tributacao labels', () => {
  it('federal CST and retencao', () => {
    expect(cstPisCofinsLabel('1')).toContain('Alíquota básica')
    expect(cstPisCofinsLabel(null)).toBe('—')
    expect(tpRetPisCofinsCsllLabel('3')).toContain('retidos')
  })

  it('issqn labels and sim/nao', () => {
    expect(tributacaoIssqnLabel('1')).toContain('tributável')
    expect(regimeEspecialTribIssqnLabel('5')).toContain('MEI')
    expect(simNaoLabel(true)).toBe('Sim')
    expect(simNaoLabel('0')).toBe('Não')
  })

  it('regime tributario', () => {
    expect(regimeTributarioLabel('SIMPLES_NACIONAL')).toContain('Simples')
    expect(regimeTributarioLabel('unknown')).toBe('unknown')
  })
})
