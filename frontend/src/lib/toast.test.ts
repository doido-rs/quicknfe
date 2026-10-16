import { describe, expect, it, vi } from 'vitest'
import { toast } from 'sonner'
import { createApiError } from '@/api/client'
import { toastError, toastFromError, toastSuccess } from './toast'

vi.mock('sonner', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}))

describe('toast helpers', () => {
  it('toastSuccess and toastError delegate to sonner', () => {
    toastSuccess('ok')
    toastError('fail')
    expect(toast.success).toHaveBeenCalledWith('ok')
    expect(toast.error).toHaveBeenCalledWith('fail')
  })

  it('toastFromError uses Error message', () => {
    toastFromError(new Error('boom'))
    expect(toast.error).toHaveBeenCalledWith('boom')
  })

  it('toastFromError uses api error message', () => {
    toastFromError(createApiError(400, 'validation'))
    expect(toast.error).toHaveBeenCalledWith('validation')
  })

  it('toastFromError falls back', () => {
    toastFromError({})
    expect(toast.error).toHaveBeenCalledWith('Operação falhou')
  })
})
