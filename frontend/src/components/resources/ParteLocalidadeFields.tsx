import type { UseFormReturn } from 'react-hook-form'
import { ResourceFieldInput } from '@/components/resources/ResourceFieldInput'
import {
  DOCUMENTO_PARTES,
  exteriorFormFields,
  LOCALIDADE_PARTE_OPTIONS,
  type LocalidadeParte,
  type ParteLocalidadeFormDef,
} from '@/lib/documentoParteLocalidade'
import type { FieldDef } from '@/resources/definitions'
import { fieldsByKeys } from '@/lib/resourceForm'

function ParteBlock({
  parte,
  allFields,
  form,
  idPrefix,
}: {
  parte: ParteLocalidadeFormDef
  allFields: FieldDef[]
  form: UseFormReturn<Record<string, string>>
  idPrefix: string
}) {
  const localidadeField = fieldsByKeys(allFields, [parte.localidadeFieldKey])[0]
  const relationField = fieldsByKeys(allFields, [parte.brasilRelationFieldKey])[0]
  const localidade = (form.watch(parte.localidadeFieldKey) || 'nao_informado') as LocalidadeParte
  const exteriorFields = exteriorFormFields(parte.exteriorPrefix).map((f) => ({
    ...f,
    required: localidade === 'exterior' && f.required,
  }))

  return (
    <div className="space-y-4 rounded-md border border-dashed p-4">
      <h3 className="text-sm font-semibold tracking-tight">{parte.title}</h3>
      {localidadeField && (
        <ResourceFieldInput
          field={{
            ...localidadeField,
            label: 'Localidade',
            enumOptions: [...LOCALIDADE_PARTE_OPTIONS],
          }}
          form={form}
          idPrefix={`${idPrefix}-loc`}
        />
      )}
      {localidade === 'brasil' && relationField && (
        <ResourceFieldInput
          field={relationField}
          form={form}
          idPrefix={`${idPrefix}-br`}
        />
      )}
      {localidade === 'exterior' && (
        <div className="grid gap-4 md:grid-cols-2">
          {exteriorFields.map((field) => (
            <ResourceFieldInput
              key={field.key}
              field={field}
              form={form}
              idPrefix={`${idPrefix}-ext`}
            />
          ))}
        </div>
      )}
    </div>
  )
}

export function ParteLocalidadeFields({
  allFields,
  form,
  sectionIndex,
}: {
  allFields: FieldDef[]
  form: UseFormReturn<Record<string, string>>
  sectionIndex: number
}) {
  return (
    <div className="space-y-6">
      {DOCUMENTO_PARTES.map((parte) => (
        <ParteBlock
          key={parte.localidadeFieldKey}
          parte={parte}
          allFields={allFields}
          form={form}
          idPrefix={`s${sectionIndex}-${parte.exteriorPrefix}`}
        />
      ))}
    </div>
  )
}
