import type { UseFormReturn } from 'react-hook-form'
import { toast } from 'sonner'
import {
  applyDocumentoFormErrors,
  validateDocumentoFormValues,
} from '@/lib/validateDocumentoForm'

/** Valida documento no client; retorna false se houver erros (toast + setError). */
export function runDocumentoFormValidation(
  form: UseFormReturn<Record<string, string>>,
): boolean {
  const values = form.getValues()
  const errors = validateDocumentoFormValues(values)
  if (errors.length === 0) return true

  form.clearErrors()
  applyDocumentoFormErrors(form.setError, errors)
  toast.error(errors[0]?.message ?? 'Corrija os campos destacados antes de salvar.')
  return false
}
