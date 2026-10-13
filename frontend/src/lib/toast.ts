import { toast as sonner } from 'sonner'
import { isApiError } from '@/api/client'

export function toastSuccess(message: string) {
  sonner.success(message)
}

export function toastError(message: string) {
  sonner.error(message)
}

export function toastFromError(error: unknown, fallback = 'Operação falhou') {
  if (error instanceof Error && error.message.trim()) {
    toastError(error.message)
    return
  }
  if (isApiError(error) && error.message.trim()) {
    toastError(error.message)
    return
  }
  toastError(fallback)
}
