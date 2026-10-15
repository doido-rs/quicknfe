import type { DateFormatPreference } from '@/lib/dateFormat'
import { formatDateForDisplay } from '@/lib/dateFormat'
import { formatBrl } from '@/lib/format'
import { fieldValue } from '@/lib/resourceForm'

const STATUS_LABELS: Record<string, string> = {
  novo: 'Novo',
  rascunho: 'Rascunho',
  assinando: 'Assinando…',
  assinado: 'Assinado',
  transmitindo: 'Transmitindo…',
  autorizada: 'Autorizada',
  autorizado: 'Autorizada',
  erro_assinatura: 'Erro na assinatura',
  erro_transmissao: 'Erro na transmissão',
  rejeitada: 'Rejeitada',
}

export function documentoStatusLabel(status: unknown): string {
  if (status == null || status === '') return '—'
  const key = String(status)
  return STATUS_LABELS[key] ?? key
}

export function documentoListDate(
  row: Record<string, unknown>,
  dateFormat: DateFormatPreference,
): string {
  const competencia = fieldValue(row, 'data_competencia')
  if (competencia != null && String(competencia).trim()) {
    return formatDateForDisplay(competencia, dateFormat)
  }
  const emissao = fieldValue(row, 'data_emissao')
  return formatDateForDisplay(emissao, dateFormat)
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
