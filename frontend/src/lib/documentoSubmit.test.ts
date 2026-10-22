import { describe, expect, it } from 'vitest'
import { prepareDocumentoSubmitBody } from './documentoSubmit'

describe('prepareDocumentoSubmitBody', () => {
  it('clears serie and numero on create when automatic', () => {
    const body = {
      serie_numero_automatico: true,
      serie: '1',
      numero_documento: 99,
    }
    prepareDocumentoSubmitBody(body, 'create')
    expect(body.serie).toBe('')
    expect(body.numero_documento).toBe(0)
  })

  it('preserves serie and numero on update when automatic', () => {
    const body = {
      serie_numero_automatico: true,
      serie: 'AUTO',
      numero_documento: 42,
    }
    prepareDocumentoSubmitBody(body, 'update')
    expect(body.serie).toBe('AUTO')
    expect(body.numero_documento).toBe(42)
  })

  it('does nothing when manual numbering', () => {
    const body = {
      serie_numero_automatico: false,
      serie: 'A',
      numero_documento: 1,
    }
    prepareDocumentoSubmitBody(body, 'create')
    expect(body.serie).toBe('A')
    expect(body.numero_documento).toBe(1)
  })
})
