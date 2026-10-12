import {
  DOCUMENTO_PARTES,
  EXTERIOR_FIELD_SUFFIXES,
  type LocalidadeParte,
  type ParteLocalidadeFormDef,
} from '@/lib/documentoParteConstants'
import { buildBody } from '@/lib/resourceForm'
import type { FieldDef } from '@/resources/definitions'

export {
  DOCUMENTO_PARTES,
  exteriorFormFields,
  LOCALIDADE_PARTE_OPTIONS,
  localidadeParteLabel,
  type LocalidadeParte,
  type ParteLocalidadeFormDef,
} from '@/lib/documentoParteConstants'

function buildExteriorObject(
  prefix: ParteLocalidadeFormDef['exteriorPrefix'],
  values: Record<string, string>,
): Record<string, string | null> | null {
  const obj: Record<string, string | null> = {}
  for (const suffix of EXTERIOR_FIELD_SUFFIXES) {
    const key = `${prefix}_${suffix}`
    const raw = values[key]?.trim() ?? ''
    obj[suffix] = raw === '' ? null : raw
  }
  if (EXTERIOR_FIELD_SUFFIXES.every((suffix) => obj[suffix] == null || obj[suffix] === '')) {
    return null
  }
  return obj
}

export function buildDocumentoBody(
  fields: FieldDef[],
  values: Record<string, string>,
): Record<string, unknown> {
  const body = buildBody(fields, values) as Record<string, unknown>

  for (const prefix of ['tomador_exterior', 'intermediario_exterior'] as const) {
    for (const suffix of EXTERIOR_FIELD_SUFFIXES) {
      delete body[`${prefix}_${suffix}`]
    }
  }

  const tomadorLoc = (values.tomador_localidade || 'brasil') as LocalidadeParte
  const intermediarioLoc = (values.intermediario_localidade ||
    'nao_informado') as LocalidadeParte

  body.tomador_localidade = tomadorLoc
  body.intermediario_localidade = intermediarioLoc

  if (tomadorLoc === 'exterior') {
    body.cliente_id = null
    body.tomador_exterior = buildExteriorObject('tomador_exterior', values)
  } else if (tomadorLoc === 'nao_informado') {
    body.cliente_id = null
    body.tomador_exterior = null
  } else {
    body.tomador_exterior = null
  }

  if (intermediarioLoc === 'exterior') {
    body.intermediario_id = null
    body.intermediario_exterior = buildExteriorObject('intermediario_exterior', values)
  } else if (intermediarioLoc === 'nao_informado') {
    body.intermediario_id = null
    body.intermediario_exterior = null
  } else {
    body.intermediario_exterior = null
  }

  if (body.modelo == null || String(body.modelo).trim() === '') {
    body.modelo = values.modelo?.trim() || 'NFSe'
  }

  const dataEmissaoRaw = values.data_emissao?.trim()
  if (body.data_emissao == null || String(body.data_emissao).trim() === '') {
    body.data_emissao = dataEmissaoRaw !== '' ? dataEmissaoRaw : new Date().toISOString()
  }

  if (body.ambiente == null) body.ambiente = Number(values.ambiente) || 2
  if (body.tipo_emissao == null) body.tipo_emissao = Number(values.tipo_emissao) || 1
  const parseAmount = (raw: string | undefined): number => {
    if (raw == null || raw.trim() === '') return 0
    const n = Number(raw)
    return Number.isFinite(n) ? n : 0
  }

  const operacao = parseAmount(values.valor_total_servicos_produtos)
  const descIncond = parseAmount(values.desconto_incondicionado)
  const descCond = parseAmount(values.desconto_condicionado)
  body.valor_total_servicos_produtos = operacao
  body.valor_liquido = Math.max(0, operacao - descIncond - descCond)

  if (body.status == null || String(body.status).trim() === '') {
    body.status = values.status?.trim() || 'novo'
  }

  for (const key of [
    'issqn_exigibilidade_suspensa',
    'issqn_retencao',
    'issqn_beneficio_municipal',
    'issqn_deducao_reducao_bc',
  ] as const) {
    const raw = values[key]
    body[key] = raw === 'true' || raw === '1'
  }

  if (body.trib_issqn == null || String(body.trib_issqn).trim() === '') {
    body.trib_issqn = values.trib_issqn?.trim() || '1'
  }
  if (body.reg_esp_trib_issqn == null || String(body.reg_esp_trib_issqn).trim() === '') {
    body.reg_esp_trib_issqn = values.reg_esp_trib_issqn?.trim() || '0'
  }

  return body
}

export function flattenDocumentoRowToForm(
  row: Record<string, unknown>,
  baseForm: Record<string, string>,
): Record<string, string> {
  const next = { ...baseForm }

  if (typeof row.tomador_localidade === 'string') {
    next.tomador_localidade = row.tomador_localidade
  }
  if (typeof row.intermediario_localidade === 'string') {
    next.intermediario_localidade = row.intermediario_localidade
  }

  for (const prefix of ['tomador_exterior', 'intermediario_exterior'] as const) {
    const ext = row[prefix]
    if (ext == null || typeof ext !== 'object') continue
    for (const suffix of EXTERIOR_FIELD_SUFFIXES) {
      const val = (ext as Record<string, unknown>)[suffix]
      next[`${prefix}_${suffix}`] = val == null ? '' : String(val)
    }
  }

  for (const key of [
    'issqn_exigibilidade_suspensa',
    'issqn_retencao',
    'issqn_beneficio_municipal',
    'issqn_deducao_reducao_bc',
  ] as const) {
    const raw = row[key]
    if (raw === true || raw === 'true' || raw === 1 || raw === '1') {
      next[key] = 'true'
    } else if (raw === false || raw === 'false' || raw === 0 || raw === '0') {
      next[key] = 'false'
    }
  }

  return next
}
