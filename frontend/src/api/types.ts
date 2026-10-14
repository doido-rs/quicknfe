export type DateFormatPreference = 'DMY' | 'MDY' | 'YMD'

export interface UserPublic {
  id: number
  email: string
  date_format?: DateFormatPreference
}

export interface CompanyPublic {
  id: number
  nome: string
}

export interface MunicipioPublic {
  id: number
  codigo_ibge: string
  nome: string
  uf: string
}

export interface EmpresaPublic {
  id: number
  cpf_cnpj: string
  razao_social: string
  regime_tributario?: string
  municipio_id?: number
  municipio?: MunicipioPublic
}

export interface CompanyMembership {
  company: CompanyPublic
  papel: string
}

export interface MeResponse {
  user: UserPublic
  companies: CompanyMembership[]
  company_ativa: CompanyPublic | null
  prestadores: EmpresaPublic[]
  prestador_ativo: EmpresaPublic | null
  papel: string | null
}
