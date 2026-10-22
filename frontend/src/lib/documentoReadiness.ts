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

  return issues
}
