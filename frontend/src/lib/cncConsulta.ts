import type { UseFormReturn } from 'react-hook-form'
import type { CncCadastroItem } from '@/api/cnc'
import type { CncLookupDef } from '@/resources/definitions'

export function applyCncItemToForm(
  form: UseFormReturn<Record<string, string>>,
  lookup: CncLookupDef,
  item: CncCadastroItem,
) {
  const set = (key: string | undefined, value: string | null | undefined) => {
    if (!key || value == null || value === '') return
    form.setValue(key, value)
  }

  set(lookup.inscricaoFieldKey, item.inscricao ?? undefined)
  set(lookup.razaoSocialFieldKey, item.razao_social ?? undefined)
  set(lookup.inscricaoMunicipalFieldKey, item.inscricao_municipal ?? undefined)
  set(lookup.emailFieldKey, item.email ?? undefined)
  set(lookup.telefoneFieldKey, item.telefone ?? undefined)
  if (lookup.municipioFieldKey && item.municipio_id != null) {
    form.setValue(lookup.municipioFieldKey, String(item.municipio_id))
  }
}

export function onlyDigits(value: string): string {
  return value.replace(/\D/g, '')
}
