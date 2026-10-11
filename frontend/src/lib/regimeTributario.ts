/** Valores alinhados ao enum `RegimeTributario` do backend (UPPER_SNAKE_CASE). */
export const REGIME_TRIBUTARIO_OPTIONS = [
  { value: 'MICROEMPREENDEDOR_INDIVIDUAL', label: 'Microempreendedor Individual (MEI)' },
  { value: 'SIMPLES_NACIONAL', label: 'Simples Nacional' },
  {
    value: 'SIMPLES_NACIONAL_EXCESSO_SUBLIMITE',
    label: 'Simples Nacional — excesso de sublimite de receita bruta',
  },
  { value: 'LUCRO_PRESUMIDO', label: 'Lucro Presumido' },
  { value: 'LUCRO_REAL', label: 'Lucro Real' },
  { value: 'LUCRO_ARBITRADO', label: 'Lucro Arbitrado' },
  { value: 'REGIME_NORMAL', label: 'Regime Normal' },
  { value: 'IMUNIDADE', label: 'Imunidade' },
  { value: 'ISENCAO', label: 'Isenção' },
] as const

export function regimeTributarioLabel(value: unknown): string {
  if (value == null || value === '') return '—'
  const found = REGIME_TRIBUTARIO_OPTIONS.find((o) => o.value === value)
  return found?.label ?? String(value)
}
