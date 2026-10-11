import { REGIME_TRIBUTARIO_OPTIONS } from '@/lib/regimeTributario'

export type FieldType = 'text' | 'number' | 'date' | 'textarea' | 'relation' | 'enum' | 'file'

/** Preenche formulário de prestador/tomador a partir da consulta CNC. */
export interface CncLookupDef {
  inscricaoFieldKey: string
  razaoSocialFieldKey?: string
  inscricaoMunicipalFieldKey?: string
  emailFieldKey?: string
  telefoneFieldKey?: string
  municipioFieldKey?: string
}

export interface FieldDef {
  key: string
  label: string
  type?: FieldType
  required?: boolean
  /** Recurso alvo para campos `relation` ou inferidos de `*_id`. */
  relationResource?: string
  /** Copia valores do registro selecionado: chave do form → chave do JSON da API. */
  relationCopyTo?: Record<string, string>
  /** Campo somente leitura na UI (ex.: derivado de relação). */
  readOnly?: boolean
  /** Opções para campos `type: 'enum'`. */
  enumOptions?: { value: string; label: string }[]
  /** Envia `empresa_id` na listagem da relação (valor = chave do formulário). */
  relationScopeFromField?: string
  /** Para `type: 'file'` (ex.: `.pfx,.p12`). */
  accept?: string
  /** Mapeia o arquivo enviado para chaves do JSON da API. */
  fileUploadAs?: { base64Key: string; nameKey: string }
}

/** Seção do formulário (layout); campos continuam em `fields` para API/CRUD. */
export type FormSectionDef =
  | {
      type: 'prestador-partes-row'
      prestadorTitle: string
      prestadorFieldKey: string
      partesTitle: string
      partesFieldKeys: string[]
    }
  | { type: 'fields'; title: string; fieldKeys: string[]; columns?: 1 | 2 }

export interface ResourceDef {
  key: string
  title: string
  /** Agrupa itens no menu lateral (ordem segue `resources`). */
  menuSection: string
  path: string
  idField?: string
  fields: FieldDef[]
  /** Layout em blocos (ex.: nota fiscal: prestador, tomador/intermediário, extras). */
  formSections?: FormSectionDef[]
  /** Sem POST (ex.: logs de retorno). */
  allowCreate?: boolean
  /** Sem DELETE na UI (padrão: permitido). */
  allowDelete?: boolean
  /** Consulta CNC por inscrição federal (CPF/CNPJ). */
  cncLookup?: CncLookupDef
  nested?: {
    label: string
    path: (parentId: number) => string
    fields: FieldDef[]
    createButtonLabel?: string
  }
}

export const resources: ResourceDef[] = [
  {
    key: 'documentos',
    title: 'Notas fiscais',
    menuSection: 'Nota fiscal',
    path: '/api/documentos',
    fields: [
      {
        key: 'empresa_id',
        label: 'Prestador de serviço',
        type: 'relation',
        relationResource: 'prestadores',
        required: true,
      },
      {
        key: 'cliente_id',
        label: 'Tomador',
        type: 'relation',
        relationResource: 'clientes',
        relationScopeFromField: 'empresa_id',
      },
      {
        key: 'intermediario_id',
        label: 'Intermediário',
        type: 'relation',
        relationResource: 'intermediarios',
        relationScopeFromField: 'empresa_id',
      },
      { key: 'modelo', label: 'Modelo', required: true },
      { key: 'serie', label: 'Série', required: true },
      { key: 'numero_documento', label: 'Número', type: 'number', required: true },
      { key: 'status', label: 'Status', required: true },
      {
        key: 'lote_id',
        label: 'Lote',
        type: 'relation',
        relationResource: 'lotes',
        relationScopeFromField: 'empresa_id',
      },
    ],
    formSections: [
      {
        type: 'prestador-partes-row',
        prestadorTitle: 'Prestador de serviço',
        prestadorFieldKey: 'empresa_id',
        partesTitle: 'Tomador e intermediário',
        partesFieldKeys: ['cliente_id', 'intermediario_id'],
      },
      {
        type: 'fields',
        title: 'Dados extras',
        fieldKeys: ['modelo', 'serie', 'numero_documento', 'status', 'lote_id'],
        columns: 1,
      },
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
    key: 'municipios',
    title: 'Municípios (IBGE)',
    menuSection: 'Referência',
    path: '/api/municipios',
    allowCreate: false,
    allowDelete: false,
    fields: [
      { key: 'nome', label: 'Nome', required: true },
      { key: 'codigo_ibge', label: 'Código IBGE', required: true },
      { key: 'uf', label: 'UF', required: true },
    ],
  },
  {
    key: 'prestadores',
    title: 'Prestadores de serviço',
    menuSection: 'Prestador e partes',
    path: '/api/prestadores',
    cncLookup: {
      inscricaoFieldKey: 'cpf_cnpj',
      razaoSocialFieldKey: 'razao_social',
      inscricaoMunicipalFieldKey: 'inscricao_municipal',
      municipioFieldKey: 'municipio_id',
    },
    fields: [
      { key: 'cpf_cnpj', label: 'CPF/CNPJ do prestador', required: true },
      { key: 'razao_social', label: 'Razão social', required: true },
      { key: 'inscricao_municipal', label: 'Inscrição municipal' },
      { key: 'inscricao_estadual', label: 'Inscrição estadual' },
      {
        key: 'regime_tributario',
        label: 'Regime tributário',
        type: 'enum',
        enumOptions: [...REGIME_TRIBUTARIO_OPTIONS],
        required: true,
      },
      {
        key: 'municipio_id',
        label: 'Município',
        type: 'relation',
        relationResource: 'municipios',
        required: true,
      },
    ],
    nested: {
      label: 'Certificados digitais',
      path: (prestadorId) => `/api/prestadores/${prestadorId}/certificados`,
      createButtonLabel: 'Enviar certificado',
      fields: [
        { key: 'tipo', label: 'Tipo (ex.: A1)', required: true },
        {
          key: 'arquivo',
          label: 'Arquivo do certificado',
          type: 'file',
          accept: '.pfx,.p12',
          required: true,
          fileUploadAs: { base64Key: 'arquivo_base64', nameKey: 'arquivo_nome' },
        },
        { key: 'senha', label: 'Senha do certificado', required: true },
        { key: 'data_validade', label: 'Validade', type: 'date', required: true },
      ],
    },
  },
  {
    key: 'clientes',
    title: 'Tomadores de serviço',
    menuSection: 'Prestador e partes',
    path: '/api/clientes',
    cncLookup: {
      inscricaoFieldKey: 'cpf_cnpj',
      razaoSocialFieldKey: 'razao_social',
      emailFieldKey: 'email',
      telefoneFieldKey: 'telefone',
      municipioFieldKey: 'municipio_id',
    },
    fields: [
      { key: 'cpf_cnpj', label: 'CPF/CNPJ do tomador', required: true },
      { key: 'razao_social', label: 'Razão social', required: true },
      { key: 'email', label: 'E-mail' },
      { key: 'telefone', label: 'Telefone' },
      {
        key: 'municipio_id',
        label: 'Município',
        type: 'relation',
        relationResource: 'municipios',
      },
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
]
