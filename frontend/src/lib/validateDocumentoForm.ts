import { parseDocumentoAmount } from '@/lib/documentoValorLiquido'

export type DocumentoFormFieldError = { field: string; message: string }

const ISO_DATE = /^\d{4}-\d{2}-\d{2}$/

function isValidIsoDate(value: string): boolean {
  if (!ISO_DATE.test(value)) return false
  const [y, m, d] = value.split('-').map(Number)
  const dt = new Date(y, m - 1, d)
  return dt.getFullYear() === y && dt.getMonth() === m - 1 && dt.getDate() === d
}

/** Validações alinhadas às regras críticas de assinatura (UX antecipada). */
export function validateDocumentoFormValues(
  values: Record<string, string>,
): DocumentoFormFieldError[] {
  const errors: DocumentoFormFieldError[] = []

  const competencia = values.data_competencia?.trim() ?? ''
  if (!competencia) {
    errors.push({ field: 'data_competencia', message: 'Informe a data de competência.' })
  } else if (!isValidIsoDate(competencia)) {
    errors.push({ field: 'data_competencia', message: 'Data de competência inválida.' })
  }

  const operacao = parseDocumentoAmount(values.valor_total_servicos_produtos)
  if (operacao <= 0) {
    errors.push({
      field: 'valor_total_servicos_produtos',
      message: 'O valor da operação deve ser maior que zero.',
    })
  }

  const tomadorLoc = values.tomador_localidade || 'nao_informado'
  if (tomadorLoc === 'brasil' && !values.cliente_id?.trim()) {
    errors.push({
      field: 'cliente_id',
      message: 'Selecione o tomador quando a localidade for Brasil.',
    })
  }

  const interLoc = values.intermediario_localidade || 'nao_informado'
  if (interLoc === 'brasil' && !values.intermediario_id?.trim()) {
    errors.push({
      field: 'intermediario_id',
      message: 'Selecione o intermediário quando a localidade for Brasil.',
    })
  }

  const suspensa = values.issqn_exigibilidade_suspensa === 'true'
  if (suspensa && !values.issqn_n_processo_suspensao?.trim()) {
    errors.push({
      field: 'issqn_n_processo_suspensao',
      message: 'Informe o número do processo quando a exigibilidade ISSQN estiver suspensa.',
    })
  }

  return errors
}

export function applyDocumentoFormErrors(
  setError: (name: string, error: { type: string; message: string }) => void,
  errors: DocumentoFormFieldError[],
): void {
  for (const err of errors) {
    setError(err.field, { type: 'manual', message: err.message })
  }
}
