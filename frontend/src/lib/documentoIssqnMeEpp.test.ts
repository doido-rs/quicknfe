import { describe, expect, it } from 'vitest'
import { omitirCamposAliquotaIssqnMeEpp } from './documentoIssqnMeEpp'

describe('documentoIssqnMeEpp', () => {
  it('omits aliquota fields for Simples without retention', () => {
    expect(
      omitirCamposAliquotaIssqnMeEpp('SIMPLES_NACIONAL', 'false', 'false'),
    ).toBe(true)
  })

  it('keeps fields when ISS retido', () => {
    expect(
      omitirCamposAliquotaIssqnMeEpp('SIMPLES_NACIONAL', 'true', 'false'),
    ).toBe(false)
  })
})
