import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import type { DashboardDailyPoint } from '@/api/dashboard'
import { formatMonthLabel } from '@/lib/format'

type MonthComparisonLineChartProps = {
  title: string
  description: string
  data: DashboardDailyPoint[]
  previousMonth: { year: number; month: number }
  currentMonth: { year: number; month: number }
  formatValue: (value: number) => string
}

export function MonthComparisonLineChart({
  title,
  description,
  data,
  previousMonth,
  currentMonth,
  formatValue,
}: MonthComparisonLineChartProps) {
  const prevLabel = formatMonthLabel(previousMonth.year, previousMonth.month)
  const currLabel = formatMonthLabel(currentMonth.year, currentMonth.month)

  const chartData = data.map((point) => ({
    day: point.day,
    previous: point.previous_month,
    current: point.current_month,
  }))

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="h-[320px] w-full">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={chartData} margin={{ top: 8, right: 8, left: 0, bottom: 0 }}>
              <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
              <XAxis
                dataKey="day"
                tickLine={false}
                axisLine={false}
                tickMargin={8}
                tickFormatter={(day) => `Dia ${day}`}
              />
              <YAxis
                tickLine={false}
                axisLine={false}
                tickMargin={8}
                width={72}
                tickFormatter={(v) => formatValue(Number(v))}
              />
              <Tooltip
                labelFormatter={(day) => `Dia ${day}`}
                content={({ active, payload, label }) => {
                  if (!active || !payload?.length) return null
                  return (
                    <div className="rounded-lg border bg-background px-3 py-2 text-xs shadow-md">
                      <p className="mb-1 font-medium">Dia {label}</p>
                      {payload.map((entry) => {
                        const key = String(entry.dataKey ?? '')
                        const series = key === 'previous' ? prevLabel : currLabel
                        return (
                          <p key={key} className="text-muted-foreground">
                            {series}:{' '}
                            <span className="font-medium text-foreground">
                              {formatValue(Number(entry.value ?? 0))}
                            </span>
                          </p>
                        )
                      })}
                    </div>
                  )
                }}
              />
              <Legend
                formatter={(value) => (value === 'previous' ? prevLabel : currLabel)}
              />
              <Line
                type="monotone"
                dataKey="previous"
                name="previous"
                stroke="var(--chart-2)"
                strokeWidth={2}
                dot={false}
                activeDot={{ r: 4 }}
              />
              <Line
                type="monotone"
                dataKey="current"
                name="current"
                stroke="var(--chart-1)"
                strokeWidth={2}
                dot={false}
                activeDot={{ r: 4 }}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  )
}
