export interface UserPublic {
  id: number
  email: string
}

export interface EmpresaPublic {
  id: number
  cpf_cnpj: string
  razao_social: string
}

export interface EmpresaMembership {
  empresa: EmpresaPublic
  papel: string
}

export interface MeResponse {
  user: UserPublic
  empresas: EmpresaMembership[]
  empresa_ativa: EmpresaPublic | null
  papel: string | null
}
