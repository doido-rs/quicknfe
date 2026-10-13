import { useQuery } from '@tanstack/react-query'
import { FileText, TrendingUp, Wallet } from 'lucide-react'
import { MonthComparisonLineChart } from '@/components/dashboard/MonthComparisonLineChart'
import { PageTitle } from '@/components/layout/PageTitle'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { dashboardApi } from '@/api/dashboard'
import { formatBrl, formatInteger, formatMonthLabel } from '@/lib/format'

function MetricCard({
  title,
  value,
  subtitle,
  icon: Icon,
}: {
  title: string
  value: string
  subtitle: string
  icon: typeof FileText
}) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <Icon className="size-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold tracking-tight">{value}</div>
        <p className="text-xs text-muted-foreground">{subtitle}</p>
      </CardContent>
    </Card>
  )
}

export function DashboardPage() {
  const { data, isLoading, isError } = useQuery({
    queryKey: ['dashboard'],
    queryFn: () => dashboardApi.get(),
  })

  if (isLoading) {
    return (
      <div className="space-y-6">
        <PageTitle>Dashboard</PageTitle>
        <Skeleton className="h-8 w-56" />
        <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <Skeleton key={i} className="h-28 w-full" />
          ))}
        </div>
        <Skeleton className="h-[380px] w-full" />
        <Skeleton className="h-[380px] w-full" />
      </div>
    )
  }

  if (isError || !data) {
    return (
      <>
        <PageTitle>Dashboard</PageTitle>
        <p className="text-sm text-destructive">Não foi possível carregar o dashboard.</p>
      </>
    )
  }

  const prevLabel = formatMonthLabel(data.previous_month.year, data.previous_month.month)
  const currLabel = formatMonthLabel(data.current_month.year, data.current_month.month)

  return (
    <div className="space-y-6">
      <PageTitle>Dashboard</PageTitle>
      <p className="text-sm text-muted-foreground">
        Comparativo entre {prevLabel} e {currLabel} (fuso horário local).
      </p>

      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          title="Notas fiscais — mês passado"
          value={formatInteger(data.previous_month.note_count)}
          subtitle={prevLabel}
          icon={FileText}
        />
        <MetricCard
          title="Notas fiscais — este mês"
          value={formatInteger(data.current_month.note_count)}
          subtitle={currLabel}
          icon={TrendingUp}
        />
        <MetricCard
          title="Faturamento — mês passado"
          value={formatBrl(data.previous_month.revenue_total)}
          subtitle={`Valor líquido · ${prevLabel}`}
          icon={Wallet}
        />
        <MetricCard
          title="Faturamento — este mês"
          value={formatBrl(data.current_month.revenue_total)}
          subtitle={`Valor líquido · ${currLabel}`}
          icon={Wallet}
        />
      </div>

      <MonthComparisonLineChart
        title="Crescimento acumulado de notas fiscais"
        description="Quantidade acumulada de notas emitidas ao longo de cada dia do mês."
        data={data.notes_cumulative_by_day}
        previousMonth={data.previous_month}
        currentMonth={data.current_month}
        formatValue={(v) => formatInteger(v)}
      />

      <MonthComparisonLineChart
        title="Crescimento acumulado de faturamento"
        description="Soma acumulada do valor líquido das notas ao longo de cada dia do mês."
        data={data.revenue_cumulative_by_day}
        previousMonth={data.previous_month}
        currentMonth={data.current_month}
        formatValue={(v) => formatBrl(v)}
      />
    </div>
  )
}
