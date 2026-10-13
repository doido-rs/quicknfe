import { formatBrl } from '@/lib/format'
import { fieldValue } from '@/lib/resourceForm'

const STATUS_LABELS: Record<string, string> = {
  novo: 'Novo',
  rascunho: 'Rascunho',
  autorizada: 'Autorizada',
  rejeitada: 'Rejeitada',
}

export function documentoStatusLabel(status: unknown): string {
  if (status == null || status === '') return '—'
  const key = String(status)
  return STATUS_LABELS[key] ?? key
}

export function documentoListDate(row: Record<string, unknown>): string {
  const competencia = fieldValue(row, 'data_competencia')
  if (competencia != null && String(competencia).trim()) {
    const s = String(competencia)
    return s.includes('T') ? s.slice(0, 10) : s
  }
  const emissao = fieldValue(row, 'data_emissao')
  if (emissao == null || emissao === '') return '—'
  const s = String(emissao)
  return s.includes('T') ? s.slice(0, 10) : s
}

export function documentoListValor(row: Record<string, unknown>): string {
  const raw = fieldValue(row, 'valor_total_servicos_produtos')
  const n = Number(raw)
  if (!Number.isFinite(n)) return '—'
  return formatBrl(n)
}

export function documentoParteListLabel(value: unknown): string {
  if (value == null || value === '') return '—'
  return String(value)
}
