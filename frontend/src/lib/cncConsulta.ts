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
  if (lookup.regimeTributarioFieldKey && item.regime_tributario_sugerido) {
    form.setValue(lookup.regimeTributarioFieldKey, item.regime_tributario_sugerido)
  }
}

/** Indica se o CNC sugere alterar o regime (evita rejeição E0160 na SEFIN). */
export function cncRegimeRequerAtualizacao(
  lookup: CncLookupDef,
  form: UseFormReturn<Record<string, string>>,
  item: CncCadastroItem,
): boolean {
  if (!lookup.regimeTributarioFieldKey || !item.regime_tributario_sugerido) {
    return false
  }
  const atual = form.getValues(lookup.regimeTributarioFieldKey)
  return atual !== item.regime_tributario_sugerido
}

export function onlyDigits(value: string): string {
  return value.replace(/\D/g, '')
}
