import {
  DOCUMENTO_PARTES,
  exteriorFormFields,
  LOCALIDADE_PARTE_OPTIONS,
} from '@/lib/documentoParteConstants'
import {
  CST_PIS_COFINS_OPTIONS,
  TP_RET_PIS_COFINS_CSLL_OPTIONS,
} from '@/lib/documentoTributacaoFederal'
import {
  REGIME_ESPECIAL_TRIB_ISSQN_OPTIONS,
  TRIBUTACAO_ISSQN_OPTIONS,
} from '@/lib/documentoTributacaoIssqn'
import { REGIME_TRIBUTARIO_OPTIONS } from '@/lib/regimeTributario'

const SIM_NAO_OPTIONS = [
  { value: 'false', label: 'Não' },
  { value: 'true', label: 'Sim' },
] as const

function documentoParteLocalidadeFields(): FieldDef[] {
  const fields: FieldDef[] = []
  for (const parte of DOCUMENTO_PARTES) {
    fields.push({
      key: parte.localidadeFieldKey,
      label: `Localidade do ${parte.title.toLowerCase()}`,
      type: 'enum',
      enumOptions: [...LOCALIDADE_PARTE_OPTIONS],
      defaultValue: parte.localidadeFieldKey === 'tomador_localidade' ? 'brasil' : 'nao_informado',
      formOnly: true,
    })
    fields.push(...exteriorFormFields(parte.exteriorPrefix))
  }
  return fields
}

export type FieldType =
  | 'text'
  | 'number'
  | 'date'
  | 'textarea'
  | 'relation'
  | 'enum'
  | 'file'
  | 'boolean'

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
  /** Valor inicial no formulário (string). */
  defaultValue?: string
  /** Oculta na ficha de detalhes (campos só do formulário). */
  formOnly?: boolean
}

/** Seção do formulário (layout); campos continuam em `fields` para API/CRUD. */
export type FormSectionDef =
  | {
      type: 'identificacao-nota'
      title: string
      competenciaFieldKey: string
      serieFieldKey: string
      numeroFieldKey: string
      automaticoFieldKey: string
      /** Checkbox “informar manualmente” (inverso de `serie_numero_automatico`). */
      manualInformLabel: string
    }
  | {
      type: 'prestador-partes-row'
      prestadorTitle: string
      prestadorFieldKey: string
      partesTitle: string
      /** Tomador/intermediário com localidade (Brasil, exterior, não informado). */
      partesLocalidade?: boolean
      partesFieldKeys?: string[]
    }
  | {
      type: 'municipios-nota'
      title: string
      emissaoFieldKey: string
      prestacaoFieldKey: string
    }
  | { type: 'fields'; title: string; fieldKeys: string[]; columns?: 1 | 2 }

export type MenuPlacement = 'sidebar' | 'settings' | 'none'

export interface NestedDef {
  label: string
  path: (parentId: number) => string
  fields: FieldDef[]
  createButtonLabel?: string
  /** Padrão: true. Use false para painéis somente leitura (ex.: logs SEFAZ). */
  allowCreate?: boolean
  /** Ajusta o JSON antes do POST (ex.: preencher `descricao` a partir da descrição do serviço). */
  prepareCreateBody?: (body: Record<string, unknown>) => Record<string, unknown>
}

export interface ResourceDef {
  key: string
  title: string
  /** Agrupa itens no menu lateral (ordem segue `resources`). */
  menuSection: string
  /** `settings` — acessível em /app/configuracoes, fora do menu principal. */
  menuPlacement?: MenuPlacement
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
  nested?: NestedDef | NestedDef[]
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
        formOnly: true,
      },
      {
        key: 'intermediario_id',
        label: 'Intermediário',
        type: 'relation',
        relationResource: 'intermediarios',
        relationScopeFromField: 'empresa_id',
        formOnly: true,
      },
      ...documentoParteLocalidadeFields(),
      {
        key: 'municipio_emissao_id',
        label: 'Município de emissão da nota',
        type: 'relation',
        relationResource: 'municipios',
        required: true,
        formOnly: true,
      },
      {
        key: 'municipio_prestacao_id',
        label: 'Município de prestação do serviço',
        type: 'relation',
        relationResource: 'municipios',
        required: true,
        formOnly: true,
      },
      { key: 'modelo', label: 'Modelo', required: true, defaultValue: 'NFSe', readOnly: true },
      {
        key: 'data_competencia',
        label: 'Data de competência',
        type: 'date',
        required: true,
      },
      {
        key: 'serie_numero_automatico',
        label: 'Série e número automáticos',
        type: 'boolean',
        defaultValue: 'true',
      },
      { key: 'serie', label: 'Série', required: true },
      { key: 'numero_documento', label: 'Número', type: 'number', required: true },
      {
        key: 'status',
        label: 'Situação',
        type: 'enum',
        defaultValue: 'novo',
        enumOptions: [
          { value: 'novo', label: 'Novo' },
          { value: 'rascunho', label: 'Rascunho' },
          { value: 'autorizada', label: 'Autorizada' },
          { value: 'rejeitada', label: 'Rejeitada' },
        ],
      },
      {
        key: 'numero_doc_responsabilidade_tecnica',
        label: 'Número do documento de responsabilidade técnica',
        formOnly: true,
      },
      { key: 'documento_referencia', label: 'Documento de referência', formOnly: true },
      {
        key: 'informacoes_complementares',
        label: 'Informações complementares',
        type: 'textarea',
        formOnly: true,
      },
      {
        key: 'numero_pedido_ordem_b2b',
        label:
          'Número do Pedido, Ordem de Compra, Ordem de Serviço ou Projeto que autorize a prestação do serviço/fornecimento em operações B2B',
        formOnly: true,
      },
      {
        key: 'ambiente',
        label: 'Ambiente',
        type: 'number',
        formOnly: true,
        defaultValue: '2',
      },
      {
        key: 'tipo_emissao',
        label: 'Tipo de emissão',
        type: 'number',
        formOnly: true,
        defaultValue: '1',
      },
      { key: 'data_emissao', label: 'Data de emissão', formOnly: true },
      {
        key: 'valor_total_servicos_produtos',
        label: 'Valor da operação/serviço prestado',
        type: 'number',
        required: true,
        formOnly: true,
      },
      {
        key: 'valor_recebido_intermediario',
        label: 'Valor recebido pelo intermediário',
        type: 'number',
        formOnly: true,
      },
      {
        key: 'desconto_incondicionado',
        label: 'Desconto incondicionado',
        type: 'number',
        formOnly: true,
        defaultValue: '0',
      },
      {
        key: 'desconto_condicionado',
        label: 'Desconto condicionado',
        type: 'number',
        formOnly: true,
        defaultValue: '0',
      },
      {
        key: 'valor_liquido',
        label: 'Valor líquido',
        type: 'number',
        formOnly: true,
        defaultValue: '0',
      },
      {
        key: 'trib_issqn',
        label: 'Tributação do ISSQN sobre o serviço prestado',
        type: 'enum',
        enumOptions: [...TRIBUTACAO_ISSQN_OPTIONS],
        required: true,
        defaultValue: '1',
        formOnly: true,
      },
      {
        key: 'reg_esp_trib_issqn',
        label: 'Regime Especial de Tributação',
        type: 'enum',
        enumOptions: [...REGIME_ESPECIAL_TRIB_ISSQN_OPTIONS],
        required: true,
        defaultValue: '0',
        formOnly: true,
      },
      {
        key: 'issqn_exigibilidade_suspensa',
        label:
          'A exigibilidade do recolhimento do ISSQN devido nesta operação está suspensa?',
        type: 'enum',
        enumOptions: [...SIM_NAO_OPTIONS],
        required: true,
        defaultValue: 'false',
        formOnly: true,
      },
      {
        key: 'issqn_retencao',
        label: 'Há retenção do ISSQN pelo Tomador ou pelo Intermediário?',
        type: 'enum',
        enumOptions: [...SIM_NAO_OPTIONS],
        required: true,
        defaultValue: 'false',
        formOnly: true,
      },
      {
        key: 'issqn_beneficio_municipal',
        label: 'Este serviço prestado está amparado por algum benefício municipal?',
        type: 'enum',
        enumOptions: [...SIM_NAO_OPTIONS],
        required: true,
        defaultValue: 'false',
        formOnly: true,
      },
      {
        key: 'issqn_deducao_reducao_bc',
        label:
          'Será aplicado algum tipo de Dedução/Redução à base de cálculo do ISSQN?',
        type: 'enum',
        enumOptions: [...SIM_NAO_OPTIONS],
        required: true,
        defaultValue: 'false',
        formOnly: true,
      },
      { key: 'issqn_aliquota', label: 'Alíquota (%)', type: 'number', formOnly: true },
      { key: 'issqn_base_calculo', label: 'BC ISSQN (R$)', type: 'number', formOnly: true },
      { key: 'issqn_valor', label: 'Valor ISSQN (R$)', type: 'number', formOnly: true },
      {
        key: 'cst_pis_cofins',
        label: 'Situação Tributária do PIS/COFINS',
        type: 'enum',
        enumOptions: [...CST_PIS_COFINS_OPTIONS],
        required: true,
        defaultValue: '00',
        formOnly: true,
      },
      {
        key: 'tp_ret_pis_cofins_csll',
        label: 'Tipo de retenção do PIS/COFINS/CSLL',
        type: 'enum',
        enumOptions: [...TP_RET_PIS_COFINS_CSLL_OPTIONS],
        required: true,
        defaultValue: '0',
        formOnly: true,
      },
      { key: 'valor_irrf', label: 'IRRF (R$)', type: 'number', formOnly: true },
      {
        key: 'valor_contrib_sociais_retidas',
        label: 'Contribuições Sociais — Retidas (R$)',
        type: 'number',
        formOnly: true,
      },
      {
        key: 'valor_contrib_previdenciaria_retida',
        label: 'Contribuição Previdenciária — Retida (R$)',
        type: 'number',
        formOnly: true,
      },
    ],
    formSections: [
      {
        type: 'identificacao-nota',
        title: 'Competência e numeração',
        competenciaFieldKey: 'data_competencia',
        serieFieldKey: 'serie',
        numeroFieldKey: 'numero_documento',
        automaticoFieldKey: 'serie_numero_automatico',
        manualInformLabel: 'Informar série e número manualmente',
      },
      {
        type: 'prestador-partes-row',
        prestadorTitle: 'Prestador de serviço',
        prestadorFieldKey: 'empresa_id',
        partesTitle: 'Tomador e intermediário',
        partesLocalidade: true,
      },
      {
        type: 'municipios-nota',
        title: 'Municípios da nota',
        emissaoFieldKey: 'municipio_emissao_id',
        prestacaoFieldKey: 'municipio_prestacao_id',
      },
      {
        type: 'fields',
        title: 'Valores do Fornecimento/Serviço Prestado',
        fieldKeys: [
          'valor_total_servicos_produtos',
          'valor_recebido_intermediario',
          'desconto_incondicionado',
          'desconto_condicionado',
        ],
        columns: 2,
      },
      {
        type: 'fields',
        title: 'Tributação Municipal',
        fieldKeys: [
          'trib_issqn',
          'reg_esp_trib_issqn',
          'issqn_exigibilidade_suspensa',
          'issqn_retencao',
          'issqn_beneficio_municipal',
          'issqn_deducao_reducao_bc',
          'issqn_aliquota',
          'issqn_base_calculo',
          'issqn_valor',
        ],
        columns: 2,
      },
      {
        type: 'fields',
        title: 'Tributação Federal',
        fieldKeys: [
          'cst_pis_cofins',
          'tp_ret_pis_cofins_csll',
          'valor_irrf',
          'valor_contrib_sociais_retidas',
          'valor_contrib_previdenciaria_retida',
        ],
        columns: 2,
      },
      {
        type: 'fields',
        title: 'Informações complementares',
        fieldKeys: [
          'numero_doc_responsabilidade_tecnica',
          'documento_referencia',
          'informacoes_complementares',
          'numero_pedido_ordem_b2b',
        ],
        columns: 2,
      },
      {
        type: 'fields',
        title: 'Modelo',
        fieldKeys: ['modelo'],
        columns: 1,
      },
    ],
    nested: [
      {
        label: 'Itens da nota',
        path: (docId) => `/api/documentos/${docId}/itens`,
        prepareCreateBody: (body) => {
          const servico = body.descricao_servico
          if (
            (body.descricao == null || String(body.descricao).trim() === '') &&
            servico != null &&
            String(servico).trim() !== ''
          ) {
            body.descricao = String(servico).trim().slice(0, 500)
          }
          return body
        },
        fields: [
          { key: 'numero_item', label: 'Nº item', type: 'number', required: true },
          {
            key: 'descricao_servico',
            label: 'Descrição do serviço/fornecimento',
            type: 'textarea',
            required: true,
          },
          {
            key: 'cna_id',
            label: 'Código de Tributação Nacional',
            type: 'relation',
            relationResource: 'cnas',
            required: true,
          },
          { key: 'quantidade', label: 'Quantidade', type: 'number', defaultValue: '1' },
          { key: 'valor_unitario', label: 'Valor unitário', type: 'number', defaultValue: '0' },
          { key: 'valor_total_bruto', label: 'Total bruto', type: 'number', defaultValue: '0' },
        ],
      },
      {
        label: 'Logs de retorno (SEFAZ)',
        path: (docId) => `/api/documentos/${docId}/logs-retorno`,
        allowCreate: false,
        fields: [
          { key: 'codigo_erro', label: 'Código erro', required: true },
          { key: 'mensagem_retorno', label: 'Mensagem', required: true },
          { key: 'correcao_sugerida', label: 'Correção sugerida' },
          { key: 'data_registro', label: 'Data registro', type: 'date' },
        ],
      },
    ],
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
    menuPlacement: 'none',
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
    key: 'cnas',
    title: 'Códigos de tributação nacional (CNA)',
    menuSection: 'Cadastros',
    menuPlacement: 'settings',
    path: '/api/cnas',
    allowCreate: false,
    allowDelete: false,
    fields: [
      { key: 'codigo', label: 'Código', required: true },
      { key: 'descricao', label: 'Descrição', required: true },
    ],
  },
  {
    key: 'municipios',
    title: 'Municípios (IBGE)',
    menuSection: 'Cadastros',
    menuPlacement: 'settings',
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
    menuSection: 'Cadastros',
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
    menuSection: 'Cadastros',
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
    menuSection: 'Cadastros',
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

export function sidebarResources(): ResourceDef[] {
  return resources.filter((r) => (r.menuPlacement ?? 'sidebar') === 'sidebar')
}

export function settingsResources(): ResourceDef[] {
  return resources.filter((r) => r.menuPlacement === 'settings')
}

export function resourceNesteds(def: ResourceDef): NestedDef[] {
  if (!def.nested) return []
  return Array.isArray(def.nested) ? def.nested : [def.nested]
}
