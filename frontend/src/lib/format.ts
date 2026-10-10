export function formatMonthLabel(year: number, month: number): string {
  const label = new Intl.DateTimeFormat('pt-BR', { month: 'long', year: 'numeric' }).format(
    new Date(year, month - 1, 1),
  )
  return label.charAt(0).toUpperCase() + label.slice(1)
}

export function formatBrl(value: number): string {
  return new Intl.NumberFormat('pt-BR', { style: 'currency', currency: 'BRL' }).format(value)
}

export function formatInteger(value: number): string {
  return new Intl.NumberFormat('pt-BR').format(value)
}
