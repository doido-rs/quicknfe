import {
  Building2,
  FileText,
  Handshake,
  Layers,
  LayoutGrid,
  ScrollText,
  Shield,
  Users,
} from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import { NavLink, useLocation } from 'react-router-dom'
import { useAuth } from '@/auth/AuthContext'
import { Badge } from '@/components/ui/badge'
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
} from '@/components/ui/sidebar'
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
  empresa: Building2,
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
  const location = useLocation()

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader className="border-b border-sidebar-border p-4">
        <div className="flex flex-col gap-1 group-data-[collapsible=icon]:items-center">
          <span className="text-sm font-semibold tracking-tight">QuickNFe</span>
          {me?.empresa_ativa && (
            <Badge variant="secondary" className="w-fit max-w-full truncate font-normal">
              {me.empresa_ativa.razao_social}
            </Badge>
          )}
        </div>
      </SidebarHeader>
      <SidebarContent>
        {resourcesByMenuSection().map((section) => (
          <SidebarGroup key={section.label}>
            <SidebarGroupLabel>{section.label}</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {section.items.map((r) => {
                  const path = `/app/${r.key}`
                  const isActive =
                    location.pathname === path || location.pathname.startsWith(`${path}/`)
                  return (
                    <SidebarMenuItem key={r.key}>
                      <SidebarMenuButton asChild isActive={isActive} tooltip={r.title}>
                        <NavLink to={path}>
                          <ResourceIcon resourceKey={r.key} />
                          <span>{r.title}</span>
                        </NavLink>
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
        <Button type="button" variant="outline" className="w-full" onClick={() => signOut()}>
          Sair
        </Button>
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
