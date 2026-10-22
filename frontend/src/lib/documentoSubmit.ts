/** Ajusta série/número automáticos antes do POST/PATCH do documento. */
export function prepareDocumentoSubmitBody(
  body: Record<string, unknown>,
  mode: 'create' | 'update',
): void {
  if (body.serie_numero_automatico !== true) return
  if (mode === 'create') {
    body.numero_documento = 0
    body.serie = ''
  }
}
