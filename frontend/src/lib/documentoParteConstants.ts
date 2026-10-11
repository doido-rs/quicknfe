export const LOCALIDADE_PARTE_OPTIONS = [
  { value: 'brasil', label: 'Brasil' },
  { value: 'exterior', label: 'Exterior' },
  { value: 'nao_informado', label: 'Não informado' },
] as const

export type LocalidadeParte = (typeof LOCALIDADE_PARTE_OPTIONS)[number]['value']

export interface ParteLocalidadeFormDef {
  title: string
  localidadeFieldKey: string
  brasilRelationFieldKey: string
  exteriorPrefix: 'tomador_exterior' | 'intermediario_exterior'
}

export const DOCUMENTO_PARTES: ParteLocalidadeFormDef[] = [
  {
    title: 'Tomador',
    localidadeFieldKey: 'tomador_localidade',
    brasilRelationFieldKey: 'cliente_id',
    exteriorPrefix: 'tomador_exterior',
  },
  {
    title: 'Intermediário',
    localidadeFieldKey: 'intermediario_localidade',
    brasilRelationFieldKey: 'intermediario_id',
    exteriorPrefix: 'intermediario_exterior',
  },
]

const EXTERIOR_FIELD_SPECS: { suffix: string; label: string; required?: boolean }[] = [
  { suffix: 'razao_social', label: 'Nome/Razão Social', required: true },
  { suffix: 'telefone', label: 'Telefone' },
  { suffix: 'email', label: 'E-mail' },
  { suffix: 'logradouro', label: 'Logradouro', required: true },
  { suffix: 'numero', label: 'Número', required: true },
  { suffix: 'complemento', label: 'Complemento' },
  { suffix: 'bairro', label: 'Bairro', required: true },
  { suffix: 'cidade', label: 'Cidade', required: true },
  { suffix: 'codigo_postal', label: 'Código de Endereçamento Postal', required: true },
  { suffix: 'estado_regiao', label: 'Estado, província ou região', required: true },
  { suffix: 'pais', label: 'País', required: true },
]

export const EXTERIOR_FIELD_SUFFIXES = EXTERIOR_FIELD_SPECS.map((s) => s.suffix)

export interface ParteExteriorFieldDef {
  key: string
  label: string
  required?: boolean
  formOnly?: boolean
}

export function exteriorFormFields(
  prefix: ParteLocalidadeFormDef['exteriorPrefix'],
): ParteExteriorFieldDef[] {
  return EXTERIOR_FIELD_SPECS.map(({ suffix, label, required }) => ({
    key: `${prefix}_${suffix}`,
    label,
    required,
    formOnly: true,
  }))
}

export function localidadeParteLabel(value: string): string {
  return LOCALIDADE_PARTE_OPTIONS.find((o) => o.value === value)?.label ?? value
}
