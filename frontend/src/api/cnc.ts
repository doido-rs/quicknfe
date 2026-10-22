import { apiFetch } from '@/api/client'

export interface CncCadastroItem {
  inscricao?: string | null
  tp_inscricao?: string | null
  razao_social?: string | null
  inscricao_municipal?: string | null
  email?: string | null
  telefone?: string | null
  codigo_municipio_ibge?: string | null
  municipio_id?: number | null
  cep?: string | null
  logradouro?: string | null
  numero?: string | null
  complemento?: string | null
  bairro?: string | null
  op_simp_nac?: number | null
  regime_tributario_sugerido?: string | null
}

export interface CncConsultaCadResponse {
  status_processamento?: string | null
  tipo_ambiente?: string | null
  data_hora_processamento?: string | null
  alertas: { codigo?: string | null; descricao?: string | null }[]
  erros: { codigo?: string | null; descricao?: string | null }[]
  itens: CncCadastroItem[]
}

export interface ConsultaCncCadParams {
  inscricaoFederal: string
  codMunicipio?: number
  indicadorMunicipal?: string
  empresaId?: number
}

function buildQuery(params: ConsultaCncCadParams): string {
  const q = new URLSearchParams()
  q.set('inscricao_federal', params.inscricaoFederal.replace(/\D/g, ''))
  if (params.codMunicipio != null) {
    q.set('cod_municipio', String(params.codMunicipio))
  }
  if (params.indicadorMunicipal?.trim()) {
    q.set('indicador_municipal', params.indicadorMunicipal.trim())
  }
  if (params.empresaId != null) {
    q.set('empresa_id', String(params.empresaId))
  }
  return q.toString()
}

export function consultaCncCad(params: ConsultaCncCadParams) {
  return apiFetch<CncConsultaCadResponse>(`/api/cnc/cad?${buildQuery(params)}`)
}
