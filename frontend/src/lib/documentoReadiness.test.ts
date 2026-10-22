import { describe, expect, it } from 'vitest'
import { documentoReadinessIssues } from './documentoReadiness'

describe('documentoReadinessIssues', () => {
  it('flags tomador brasil without cliente', () => {
    const issues = documentoReadinessIssues(
      { status: 'novo', tomador_localidade: 'brasil', cliente_id: null },
      1,
    )
    expect(issues.some((i) => i.id === 'tomador')).toBe(true)
  })

  it('flags missing itens', () => {
    const issues = documentoReadinessIssues(
      { status: 'novo', tomador_localidade: 'nao_informado' },
      0,
    )
    expect(issues.some((i) => i.id === 'itens')).toBe(true)
  })

  it('warns trib issqn nao incidencia with itens', () => {
    const issues = documentoReadinessIssues(
      { status: 'novo', tomador_localidade: 'nao_informado', trib_issqn: 'NAO_INCIDENCIA' },
      2,
    )
    expect(issues.some((i) => i.id === 'e0539-trib-issqn')).toBe(true)
  })
})
