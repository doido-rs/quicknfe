import { screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { renderWithProviders } from '../test/renderWithProviders'
import { stubApiFetch } from '../test/mockFetch'

describe('DashboardPage', () => {
  it('shows summary cards and chart titles', async () => {
    stubApiFetch({
      '/api/me': {
        user: { id: 1, email: 'u@test.com' },
        companies: [],
        company_ativa: { id: 1, nome: 'Company' },
        prestadores: [],
        prestador_ativo: { id: 1, cpf_cnpj: '1', razao_social: 'Minha Empresa' },
        papel: 'proprietario',
      },
      '/api/dashboard': {
        previous_month: { year: 2026, month: 9, note_count: 12, revenue_total: 1500 },
        current_month: { year: 2026, month: 10, note_count: 5, revenue_total: 800 },
        notes_cumulative_by_day: [{ day: 1, previous_month: 1, current_month: 1 }],
        revenue_cumulative_by_day: [{ day: 1, previous_month: 100, current_month: 200 }],
      },
    })

    vi.stubGlobal(
      'ResizeObserver',
      class {
        observe() {}
        unobserve() {}
        disconnect() {}
      },
    )

    await renderWithProviders({ route: '/app/dashboard' })

    expect(await screen.findByRole('heading', { name: /^dashboard$/i })).toBeInTheDocument()
    expect(screen.getByText('12')).toBeInTheDocument()
    expect(screen.getByText('5')).toBeInTheDocument()
    expect(screen.getByText(/Crescimento acumulado de notas fiscais/i)).toBeInTheDocument()
    expect(screen.getByText(/Crescimento acumulado de faturamento/i)).toBeInTheDocument()
  })
})
