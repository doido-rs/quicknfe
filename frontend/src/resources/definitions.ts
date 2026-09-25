export type FieldType = 'text' | 'number' | 'date' | 'textarea'

export interface FieldDef {
  key: string
  label: string
  type?: FieldType
  required?: boolean
}

export interface ResourceDef {
  key: string
  title: string
  path: string
  idField?: string
  fields: FieldDef[]
  /** Sem POST (ex.: logs de retorno). */
  allowCreate?: boolean
  nested?: {
    label: string
    path: (parentId: number) => string
    fields: FieldDef[]
  }
}

export const resources: ResourceDef[] = [
  {
    key: 'empresa',
    title: 'Empresa',
    path: '/api/empresa',
    fields: [
      { key: 'cpf_cnpj', label: 'CPF/CNPJ', required: true },
      { key: 'razao_social', label: 'Razão social', required: true },
      { key: 'inscricao_municipal', label: 'Inscrição municipal' },
      { key: 'inscricao_estadual', label: 'Inscrição estadual' },
      { key: 'regime_tributario', label: 'Regime tributário', type: 'number' },
      { key: 'codigo_municipio_ibge', label: 'Município IBGE' },
      { key: 'uf', label: 'UF' },
    ],
  },
  {
    key: 'clientes',
    title: 'Clientes',
    path: '/api/clientes',
    fields: [
      { key: 'cpf_cnpj', label: 'CPF/CNPJ', required: true },
      { key: 'razao_social', label: 'Razão social', required: true },
      { key: 'email', label: 'E-mail' },
      { key: 'telefone', label: 'Telefone' },
      { key: 'uf', label: 'UF' },
    ],
  },
  {
    key: 'certificados',
    title: 'Certificados',
    path: '/api/certificados',
    fields: [
      { key: 'tipo', label: 'Tipo', required: true },
      { key: 'caminho_arquivo', label: 'Caminho arquivo' },
      { key: 'senha', label: 'Senha (criar/trocar)' },
      { key: 'data_validade', label: 'Validade', type: 'date', required: true },
    ],
  },
  {
    key: 'lotes',
    title: 'Lotes',
    path: '/api/lotes',
    fields: [
      { key: 'numero_lote', label: 'Número lote', type: 'number', required: true },
      { key: 'status_lote', label: 'Status', required: true },
      { key: 'ambiente', label: 'Ambiente', type: 'number' },
      { key: 'protocolo_recebimento', label: 'Protocolo' },
    ],
  },
  {
    key: 'documentos',
    title: 'Documentos',
    path: '/api/documentos',
    fields: [
      { key: 'modelo', label: 'Modelo', required: true },
      { key: 'serie', label: 'Série', required: true },
      { key: 'numero_documento', label: 'Número', type: 'number', required: true },
      { key: 'status', label: 'Status', required: true },
      { key: 'cliente_id', label: 'Cliente ID', type: 'number' },
      { key: 'lote_id', label: 'Lote ID', type: 'number' },
    ],
    nested: {
      label: 'Itens',
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
    key: 'logs-retorno',
    title: 'Logs retorno',
    path: '/api/logs-retorno',
    allowCreate: false,
    fields: [
      { key: 'codigo_erro', label: 'Código erro', required: true },
      { key: 'mensagem_retorno', label: 'Mensagem', required: true },
      { key: 'documento_fiscal_id', label: 'Documento ID', type: 'number' },
      { key: 'lote_id', label: 'Lote ID', type: 'number' },
    ],
  },
]
