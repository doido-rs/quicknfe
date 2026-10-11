export interface UserPublic {
  id: number
  email: string
}

export interface CompanyPublic {
  id: number
  nome: string
}

export interface EmpresaPublic {
  id: number
  cpf_cnpj: string
  razao_social: string
  regime_tributario?: string
  codigo_municipio_ibge?: string
  municipio_nome?: string
  uf?: string
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
