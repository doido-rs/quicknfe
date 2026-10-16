import { describe, expect, it } from 'vitest'
import { documentoProcessamentoInFlight, documentoTerminalError } from './documentoProcessamento'

describe('documentoProcessamento', () => {
  it('detects in-flight statuses', () => {
    expect(documentoProcessamentoInFlight('assinando')).toBe(true)
    expect(documentoProcessamentoInFlight('Transmitindo')).toBe(true)
    expect(documentoProcessamentoInFlight('autorizada')).toBe(false)
  })

  it('detects terminal errors', () => {
    expect(documentoTerminalError('erro_assinatura')).toBe(true)
    expect(documentoTerminalError('erro_transmissao')).toBe(true)
    expect(documentoTerminalError('novo')).toBe(false)
  })
})
