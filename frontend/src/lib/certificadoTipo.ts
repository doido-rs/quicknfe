/** Valores permitidos — espelha CHECK em `certificados_digitais.tipo`. */
export const CERTIFICADO_TIPO_VALUES = ['A1', 'A3', 'PFX', 'PEM'] as const
export type CertificadoTipo = (typeof CERTIFICADO_TIPO_VALUES)[number]

export const CERTIFICADO_TIPO_OPTIONS: { value: CertificadoTipo; label: string }[] = [
  { value: 'A1', label: 'A1 (arquivo .pfx / .p12)' },
  { value: 'A3', label: 'A3 (token/cartão — referência)' },
  { value: 'PFX', label: 'PFX' },
  { value: 'PEM', label: 'PEM' },
]

export function isCertificadoTipo(value: string): value is CertificadoTipo {
  return (CERTIFICADO_TIPO_VALUES as readonly string[]).includes(value)
}
