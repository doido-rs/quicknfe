import { describe, expect, it } from 'vitest'
import {
  applyItensToDocumentoBody,
  operacaoFormValueFromItens,
  sumDocumentoItensValor,
} from './documentoItens'

describe('documentoItens', () => {
  it('sums valor_total_bruto', () => {
    expect(
      sumDocumentoItensValor([
        { valor_total_bruto: 100 },
        { valor_total_bruto: 50.5 },
        { valor_total_bruto: 'bad' },
      ]),
    ).toBe(150.5)
  })

  it('updates document body from draft itens', () => {
    const body: Record<string, unknown> = {
      valor_total_servicos_produtos: 0,
      valor_liquido: 0,
    }
    applyItensToDocumentoBody(
      body,
      { desconto_incondicionado: '10', desconto_condicionado: '5' },
      [{ valor_total_bruto: 200 }],
    )
    expect(body.valor_total_servicos_produtos).toBe(200)
    expect(body.valor_liquido).toBe(185)
  })

  it('derives operacao form value from itens', () => {
    expect(operacaoFormValueFromItens([], 100)).toBeNull()
    expect(
      operacaoFormValueFromItens([{ valor_total_bruto: 80 }, { valor_total_bruto: 20 }], 100),
    ).toEqual({ value: '100.00', shouldDirty: false })
    expect(
      operacaoFormValueFromItens([{ valor_total_bruto: 150 }], 100),
    ).toEqual({ value: '150.00', shouldDirty: true })
  })
})
