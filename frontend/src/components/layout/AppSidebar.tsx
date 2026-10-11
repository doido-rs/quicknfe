import { Link, useRouterState } from '@tanstack/react-router'
import {
  Building2,
  FileText,
  Handshake,
  Layers,
  LayoutDashboard,
  LayoutGrid,
  LogOut,
  MapPin,
  ScrollText,
  Shield,
  Users,
} from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import { useAuth } from '@/auth/AuthContext'
import { Button } from '@/components/ui/button'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
  useSidebar,
} from '@/components/ui/sidebar'
import { appDashboardTo, resourceListTo } from '@/lib/resourcePaths'
import { resources, type ResourceDef } from '@/resources/definitions'

function resourcesByMenuSection(): { label: string; items: ResourceDef[] }[] {
  const sections: { label: string; items: ResourceDef[] }[] = []
  for (const r of resources) {
    let section = sections.find((s) => s.label === r.menuSection)
    if (!section) {
      section = { label: r.menuSection, items: [] }
      sections.push(section)
    }
    section.items.push(r)
  }
  return sections
}

const resourceIcons: Record<string, LucideIcon> = {
  municipios: MapPin,
  prestadores: Building2,
  clientes: Users,
  intermediarios: Handshake,
  certificados: Shield,
  lotes: Layers,
  documentos: FileText,
  'logs-retorno': ScrollText,
}

function ResourceIcon({ resourceKey }: { resourceKey: string }) {
  const Icon = resourceIcons[resourceKey] ?? LayoutGrid
  return <Icon />
}

export function AppSidebar() {
  const { me, signOut } = useAuth()
  const { state, isMobile } = useSidebar()
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const iconOnly = state === 'collapsed' && !isMobile

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader className="border-b border-sidebar-border">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              size="lg"
              tooltip="QuickNFe"
              className="pointer-events-none h-auto py-2 group-data-[collapsible=icon]:size-8! group-data-[collapsible=icon]:py-0"
            >
              <div className="flex aspect-square size-8 shrink-0 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
                <span className="text-xs font-bold tracking-tight">QN</span>
              </div>
              <div className="grid min-w-0 flex-1 gap-0.5 text-left leading-tight group-data-[collapsible=icon]:hidden">
                <span className="truncate text-sm font-semibold tracking-tight">QuickNFe</span>
                {me?.prestador_ativo && (
                  <span className="truncate text-xs font-normal text-sidebar-foreground/80">
                    {me.prestador_ativo.razao_social}
                  </span>
                )}
                {me?.company_ativa && (
                  <span className="truncate text-xs text-muted-foreground">
                    {me.company_ativa.nome}
                  </span>
                )}
              </div>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Visão geral</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton
                  asChild
                  isActive={pathname === '/app/dashboard' || pathname === '/app'}
                  tooltip="Dashboard"
                >
                  <Link {...appDashboardTo}>
                    <LayoutDashboard />
                    <span>Dashboard</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        {resourcesByMenuSection().map((section) => (
          <SidebarGroup key={section.label}>
            <SidebarGroupLabel>{section.label}</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {section.items.map((r) => {
                  const path = `/app/${r.key}`
                  const isActive = pathname === path || pathname.startsWith(`${path}/`)
                  const listTo = resourceListTo(r)
                  return (
                    <SidebarMenuItem key={r.key}>
                      <SidebarMenuButton asChild isActive={isActive} tooltip={r.title}>
                        <Link {...listTo}>
                          <ResourceIcon resourceKey={r.key} />
                          <span>{r.title}</span>
                        </Link>
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                  )
                })}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        ))}
      </SidebarContent>
      <SidebarFooter className="border-t border-sidebar-border p-2">
        <Button
          type="button"
          variant="outline"
          className={iconOnly ? 'size-8 p-0' : 'w-full'}
          onClick={() => signOut()}
          title={iconOnly ? 'Sair' : undefined}
        >
          {iconOnly ? <LogOut className="size-4" /> : 'Sair'}
        </Button>
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
