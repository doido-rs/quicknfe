import { describe, expect, it, vi } from 'vitest'
import { applyCncItemToForm, onlyDigits } from './cncConsulta'

describe('onlyDigits', () => {
  it('strips non-digits', () => {
    expect(onlyDigits('12.345.678/0001-90')).toBe('12345678000190')
  })
})

describe('applyCncItemToForm', () => {
  it('sets mapped fields when present', () => {
    const setValue = vi.fn()
    const form = { setValue } as unknown as import('react-hook-form').UseFormReturn<
      Record<string, string>
    >
    applyCncItemToForm(
      form,
      {
        inscricaoFieldKey: 'cpf_cnpj',
        razaoSocialFieldKey: 'razao_social',
        emailFieldKey: 'email',
      },
      {
        inscricao: '12345678000190',
        razao_social: 'ACME',
        email: 'a@b.com',
      },
    )
    expect(setValue).toHaveBeenCalledWith('cpf_cnpj', '12345678000190')
    expect(setValue).toHaveBeenCalledWith('razao_social', 'ACME')
    expect(setValue).toHaveBeenCalledWith('email', 'a@b.com')
  })
})
