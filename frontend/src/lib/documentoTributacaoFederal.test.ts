import { describe, expect, it } from 'vitest'
import {
  cstPisCofinsToFormValue,
  cstPisCofinsLabel,
  tpRetPisCofinsCsllToFormValue,
  tpRetPisCofinsCsllLabel,
} from './documentoTributacaoFederal'

describe('documentoTributacaoFederal wire → form', () => {
  it('maps cst_pis_cofins API enum to select value', () => {
    expect(cstPisCofinsToFormValue('NENHUM')).toBe('00')
    expect(cstPisCofinsToFormValue('ALIQUOTA_BASICA')).toBe('01')
    expect(cstPisCofinsLabel('COM_SUSPENSAO')).toContain('09')
  })

  it('maps tp_ret_pis_cofins_csll API enum to select value', () => {
    expect(tpRetPisCofinsCsllToFormValue('NAO_RETIDOS')).toBe('0')
    expect(tpRetPisCofinsCsllToFormValue('RETIDOS')).toBe('3')
    expect(tpRetPisCofinsCsllLabel('PIS_RETIDO')).toContain('5')
  })
})
