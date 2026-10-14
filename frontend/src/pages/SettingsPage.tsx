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
import { PageTitle } from '@/components/layout/PageTitle'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { DATE_FORMAT_OPTIONS } from '@/lib/dateFormat'
import { resourceListTo } from '@/lib/resourcePaths'
import { settingsResources } from '@/resources/definitions'
import { useDateFormat } from '@/theme/DateFormatContext'

const settingsIcons: Record<string, LucideIcon> = {
  prestadores: Building2,
  clientes: Users,
  intermediarios: Handshake,
  municipios: MapPin,
  cnas: ScrollText,
}

export function SettingsPage() {
  const items = settingsResources()
  const { preference, setPreference } = useDateFormat()

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <PageTitle>Configurações</PageTitle>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Preferências</CardTitle>
          <CardDescription>
            Formato de data usado em formulários, listas e detalhes em todo o sistema.
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-2 max-w-sm">
          <Label htmlFor="date-format-pref">Formato de data</Label>
          <select
            id="date-format-pref"
            className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
            value={preference}
            onChange={(e) =>
              setPreference(e.target.value as (typeof DATE_FORMAT_OPTIONS)[number]['value'])
            }
          >
            {DATE_FORMAT_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </CardContent>
      </Card>

      <p className="text-sm text-muted-foreground">
        Tabelas de referência: CNA e municípios (IBGE).
      </p>
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
