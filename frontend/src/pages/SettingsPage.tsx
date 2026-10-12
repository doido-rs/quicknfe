import { Link } from '@tanstack/react-router'
import {
  Building2,
  ChevronRight,
  Handshake,
  MapPin,
  ScrollText,
  Users,
  type LucideIcon,
} from 'lucide-react'
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { resourceListTo } from '@/lib/resourcePaths'
import { settingsResources } from '@/resources/definitions'

const settingsIcons: Record<string, LucideIcon> = {
  prestadores: Building2,
  clientes: Users,
  intermediarios: Handshake,
  municipios: MapPin,
  cnas: ScrollText,
}

export function SettingsPage() {
  const items = settingsResources()

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Configurações</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Cadastros de prestadores, tomadores, intermediários, CNA e municípios (IBGE).
        </p>
      </div>
      <ul className="grid gap-3">
        {items.map((r) => {
          const Icon = settingsIcons[r.key]
          return (
            <li key={r.key}>
              <Link {...resourceListTo(r)} className="block">
                <Card className="transition-colors hover:bg-muted/40">
                  <CardHeader className="flex flex-row items-center gap-3 space-y-0 py-4">
                    {Icon && (
                      <div className="flex size-10 shrink-0 items-center justify-center rounded-lg bg-muted">
                        <Icon className="size-5 text-muted-foreground" />
                      </div>
                    )}
                    <div className="min-w-0 flex-1">
                      <CardTitle className="text-base">{r.title}</CardTitle>
                      <CardDescription className="truncate">
                        Gerenciar {r.title.toLowerCase()}
                      </CardDescription>
                    </div>
                    <ChevronRight className="size-5 shrink-0 text-muted-foreground" />
                  </CardHeader>
                </Card>
              </Link>
            </li>
          )
        })}
      </ul>
    </div>
  )
}
