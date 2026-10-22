import { describe, expect, it } from 'vitest'
import { buildDocumentoEditTabOrder, DOCUMENTO_EDIT_ITENS_TAB } from './documentoEditTabs'

describe('buildDocumentoEditTabOrder', () => {
  it('places Itens before Valores', () => {
    const order = buildDocumentoEditTabOrder(true)
    const values = order.findIndex((e) => e.kind === 'section' && e.value === 'valores')
    const itens = order.findIndex((e) => e.kind === 'itens')
    expect(itens).toBeGreaterThan(-1)
    expect(values).toBeGreaterThan(itens)
    expect(order[itens]?.value).toBe(DOCUMENTO_EDIT_ITENS_TAB)
  })

  it('omits Itens when disabled', () => {
    const order = buildDocumentoEditTabOrder(false)
    expect(order.some((e) => e.kind === 'itens')).toBe(false)
  })
})
