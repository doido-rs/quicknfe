import { apiFetch } from './client'

export type DashboardMonthSummary = {
  year: number
  month: number
  note_count: number
  revenue_total: number
}

export type DashboardDailyPoint = {
  day: number
  previous_month: number
  current_month: number
}

export type DashboardStats = {
  previous_month: DashboardMonthSummary
  current_month: DashboardMonthSummary
  notes_cumulative_by_day: DashboardDailyPoint[]
  revenue_cumulative_by_day: DashboardDailyPoint[]
}

export const dashboardApi = {
  get: () => apiFetch<DashboardStats>('/api/dashboard'),
}
