import { tribIssqnToFormValue } from '@/lib/documentoTributacaoIssqn'

export type DocumentoReadinessIssue = {
  id: string
  message: string
}

/** Heurística de UI — não substitui validação DPS na assinatura. */
export function documentoReadinessIssues(
  row: Record<string, unknown>,
  itensCount?: number,
): DocumentoReadinessIssue[] {
  const issues: DocumentoReadinessIssue[] = []
  const status = typeof row.status === 'string' ? row.status : ''
  if (status === 'autorizada' || status === 'rejeitada') return issues

  const tomadorLoc =
    typeof row.tomador_localidade === 'string' ? row.tomador_localidade : 'nao_informado'
  if (tomadorLoc === 'brasil' && (row.cliente_id == null || row.cliente_id === '')) {
    issues.push({
      id: 'tomador',
      message: 'Tomador no Brasil exige cadastro de tomador vinculado.',
    })
  }

  const interLoc =
    typeof row.intermediario_localidade === 'string'
      ? row.intermediario_localidade
      : 'nao_informado'
  if (interLoc === 'brasil' && (row.intermediario_id == null || row.intermediario_id === '')) {
    issues.push({
      id: 'intermediario',
      message: 'Intermediário no Brasil exige cadastro vinculado.',
    })
  }

  if (itensCount != null && itensCount < 1) {
    issues.push({
      id: 'itens',
      message: 'Inclua ao menos um item de serviço na edição da nota.',
    })
  }

  if (tribIssqnToFormValue(row.trib_issqn) === '4' && (itensCount ?? 0) > 0) {
    issues.push({
      id: 'e0539-trib-issqn',
      message:
        'Tributação ISSQN "Não incidência" só é válida se os CNA dos itens não forem incidentes no município de incidência (rejeição E0539 na transmissão).',
    })
  }

  return issues
}
