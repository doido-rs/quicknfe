import { Navigate, Outlet } from '@tanstack/react-router'
import { useAuth } from '@/auth/AuthContext'
import { AppSidebar } from '@/components/layout/AppSidebar'
import { ThemeToggle } from '@/components/ThemeToggle'
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar'
import { TooltipProvider } from '@/components/ui/tooltip'

export function AppLayout() {
  const { me, isLoading } = useAuth()

  if (!isLoading && !me) {
    return <Navigate to="/login" replace />
  }

  if (!isLoading && me && !me.company_ativa) {
    return <Navigate to="/tenant" replace />
  }

  if (!isLoading && me && !me.prestador_ativo) {
    return <Navigate to="/tenant/prestador" replace />
  }

  return (
    <TooltipProvider>
      <SidebarProvider>
        <AppSidebar />
        <SidebarInset>
          <header className="flex h-14 shrink-0 items-center gap-3 border-b px-4">
            <SidebarTrigger />
            <span className="font-semibold">QuickNFe</span>
            <div className="ml-auto">
              <ThemeToggle />
            </div>
          </header>
          <main className="flex-1 p-4 md:p-6">
            <Outlet />
          </main>
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  )
}
