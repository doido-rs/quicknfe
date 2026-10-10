export type FieldType = 'text' | 'number' | 'date' | 'textarea' | 'relation'

export interface FieldDef {
  key: string
  label: string
  type?: FieldType
  required?: boolean
  /** Recurso alvo para campos `relation` ou inferidos de `*_id`. */
  relationResource?: string
}

export interface ResourceDef {
  key: string
  title: string
  /** Agrupa itens no menu lateral (ordem segue `resources`). */
  menuSection: string
  path: string
  idField?: string
  fields: FieldDef[]
  /** Sem POST (ex.: logs de retorno). */
  allowCreate?: boolean
  /** Sem DELETE na UI (padrão: permitido). */
  allowDelete?: boolean
  nested?: {
    label: string
    path: (parentId: number) => string
    fields: FieldDef[]
  }
}

export const resources: ResourceDef[] = [
  {
    key: 'documentos',
    title: 'Notas fiscais',
    menuSection: 'Nota fiscal',
    path: '/api/documentos',
    fields: [
      { key: 'modelo', label: 'Modelo', required: true },
      { key: 'serie', label: 'Série', required: true },
      { key: 'numero_documento', label: 'Número', type: 'number', required: true },
      { key: 'status', label: 'Status', required: true },
      { key: 'cliente_id', label: 'Tomador', type: 'relation', relationResource: 'clientes' },
      {
        key: 'intermediario_id',
        label: 'Intermediário',
        type: 'relation',
        relationResource: 'intermediarios',
      },
      { key: 'lote_id', label: 'Lote', type: 'relation', relationResource: 'lotes' },
    ],
    nested: {
      label: 'Itens da nota',
      path: (docId) => `/api/documentos/${docId}/itens`,
      fields: [
        { key: 'numero_item', label: 'Nº item', type: 'number', required: true },
        { key: 'descricao', label: 'Descrição', required: true },
        { key: 'quantidade', label: 'Quantidade', type: 'number' },
        { key: 'valor_unitario', label: 'Valor unit.', type: 'number' },
        { key: 'valor_total_bruto', label: 'Total bruto', type: 'number' },
      ],
    },
  },
  {
    key: 'lotes',
    title: 'Lotes de transmissão',
    menuSection: 'Nota fiscal',
    path: '/api/lotes',
    fields: [
      { key: 'numero_lote', label: 'Número lote', type: 'number', required: true },
      { key: 'status_lote', label: 'Status', required: true },
      { key: 'ambiente', label: 'Ambiente', type: 'number' },
      { key: 'protocolo_recebimento', label: 'Protocolo' },
    ],
  },
  {
    key: 'logs-retorno',
    title: 'Logs de retorno (SEFAZ)',
    menuSection: 'Nota fiscal',
    path: '/api/logs-retorno',
    allowCreate: false,
    fields: [
      { key: 'codigo_erro', label: 'Código erro', required: true },
      { key: 'mensagem_retorno', label: 'Mensagem', required: true },
      {
        key: 'documento_fiscal_id',
        label: 'Nota fiscal',
        type: 'relation',
        relationResource: 'documentos',
      },
      { key: 'lote_id', label: 'Lote', type: 'relation', relationResource: 'lotes' },
    ],
  },
  {
    key: 'prestadores',
    title: 'Prestadores de serviço',
    menuSection: 'Prestador e partes',
    path: '/api/prestadores',
    fields: [
      { key: 'cpf_cnpj', label: 'CPF/CNPJ do prestador', required: true },
      { key: 'razao_social', label: 'Razão social', required: true },
      { key: 'inscricao_municipal', label: 'Inscrição municipal' },
      { key: 'inscricao_estadual', label: 'Inscrição estadual' },
      { key: 'regime_tributario', label: 'Regime tributário', type: 'number', required: true },
      { key: 'codigo_municipio_ibge', label: 'Município IBGE', required: true },
      { key: 'uf', label: 'UF', required: true },
    ],
  },
  {
    key: 'clientes',
    title: 'Tomadores de serviço',
    menuSection: 'Prestador e partes',
    path: '/api/clientes',
    fields: [
      { key: 'cpf_cnpj', label: 'CPF/CNPJ do tomador', required: true },
      { key: 'razao_social', label: 'Razão social', required: true },
      { key: 'email', label: 'E-mail' },
      { key: 'telefone', label: 'Telefone' },
      { key: 'uf', label: 'UF' },
    ],
  },
  {
    key: 'intermediarios',
    title: 'Intermediários',
    menuSection: 'Prestador e partes',
    path: '/api/intermediarios',
    fields: [
      { key: 'cpf_cnpj', label: 'CPF/CNPJ do intermediário', required: true },
      { key: 'razao_social', label: 'Razão social', required: true },
      { key: 'inscricao_municipal', label: 'Inscrição municipal' },
      { key: 'email', label: 'E-mail' },
      { key: 'telefone', label: 'Telefone' },
    ],
  },
  {
    key: 'certificados',
    title: 'Certificados digitais',
    menuSection: 'Prestador e partes',
    path: '/api/certificados',
    fields: [
      { key: 'tipo', label: 'Tipo', required: true },
      { key: 'caminho_arquivo', label: 'Caminho arquivo' },
      { key: 'senha', label: 'Senha (criar/trocar)' },
      { key: 'data_validade', label: 'Validade', type: 'date', required: true },
    ],
  },
]
