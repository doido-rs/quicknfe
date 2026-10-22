import { describe, expect, it } from 'vitest'
import { computeDocumentoValorLiquido } from './documentoValorLiquido'

describe('computeDocumentoValorLiquido', () => {
  it('subtracts descontos from operacao', () => {
    expect(
      computeDocumentoValorLiquido({
        valor_total_servicos_produtos: '1000',
        desconto_incondicionado: '100',
        desconto_condicionado: '50',
      }),
    ).toBe(850)
  })

  it('never returns negative', () => {
    expect(
      computeDocumentoValorLiquido({
        valor_total_servicos_produtos: '10',
        desconto_incondicionado: '100',
        desconto_condicionado: '0',
      }),
    ).toBe(0)
  })
})
