import type { ReactNode } from 'react'
import { ThemeToggle } from '@/components/ThemeToggle'

export function AuthPageShell({ children }: { children: ReactNode }) {
  return (
    <div className="relative flex min-h-svh items-center justify-center bg-background p-4">
      <div className="absolute right-4 top-4">
        <ThemeToggle />
      </div>
      {children}
    </div>
  )
}
