const IN_FLIGHT = new Set(['assinando', 'transmitindo'])

export function documentoProcessamentoInFlight(status: unknown): boolean {
  return IN_FLIGHT.has(String(status ?? '').toLowerCase())
}

export function documentoTerminalError(status: unknown): boolean {
  const s = String(status ?? '').toLowerCase()
  return s === 'erro_assinatura' || s === 'erro_transmissao'
}
