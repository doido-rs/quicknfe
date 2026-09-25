import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, beforeEach } from 'vitest'
import { ThemeToggle } from '@/components/ThemeToggle'
import { ThemeProvider } from './ThemeContext'

beforeEach(() => {
  localStorage.clear()
  document.documentElement.classList.remove('dark')
})

describe('ThemeToggle', () => {
  it('toggles dark class on document', async () => {
    const user = userEvent.setup()
    render(
      <ThemeProvider>
        <ThemeToggle />
      </ThemeProvider>,
    )

    expect(document.documentElement.classList.contains('dark')).toBe(false)
    await user.click(screen.getByRole('button', { name: /tema escuro/i }))
    expect(document.documentElement.classList.contains('dark')).toBe(true)
    expect(localStorage.getItem('quicknfe-theme')).toBe('dark')
    await user.click(screen.getByRole('button', { name: /tema claro/i }))
    expect(document.documentElement.classList.contains('dark')).toBe(false)
  })
})
