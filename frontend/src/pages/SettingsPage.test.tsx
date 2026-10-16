import { screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { renderWithProviders } from '../test/renderWithProviders'

const setPreference = vi.fn()

vi.mock('@/theme/DateFormatContext', () => ({
  useDateFormat: () => ({ preference: 'DMY', setPreference }),
  DateFormatProvider: ({ children }: { children: React.ReactNode }) => children,
}))

describe('SettingsPage', () => {
  it('renders preferences and settings resource links', async () => {
    await renderWithProviders({ route: '/app/configuracoes' })
    expect(await screen.findByText('Preferências')).toBeInTheDocument()
    expect(screen.getByLabelText(/formato de data/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/formato de data/i)).toHaveValue('DMY')
  })
})
