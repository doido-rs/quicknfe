/** Abas da edição da nota → índices em `documentos.formSections`. */
export const DOCUMENTO_EDIT_SECTION_TABS = [
  { value: 'identificacao', label: 'Competência', sectionIndexes: [0, 7] },
  { value: 'partes', label: 'Partes', sectionIndexes: [1, 2] },
  { value: 'valores', label: 'Valores', sectionIndexes: [3] },
  { value: 'tributos', label: 'Tributos', sectionIndexes: [4, 5] },
  { value: 'complementos', label: 'Complementos', sectionIndexes: [6] },
] as const

export const DOCUMENTO_EDIT_ITENS_TAB = 'itens' as const

/** Aba Itens fica imediatamente antes desta aba de seções (Valores). */
export const DOCUMENTO_EDIT_ITENS_INSERT_BEFORE = 'valores' as const

export type DocumentoEditTabEntry =
  | { kind: 'section'; value: string; label: string; sectionIndexes: readonly number[] }
  | { kind: 'itens'; value: typeof DOCUMENTO_EDIT_ITENS_TAB; label: string }

export function buildDocumentoEditTabOrder(
  includeItens: boolean,
  itensLabel = 'Itens',
): DocumentoEditTabEntry[] {
  const entries: DocumentoEditTabEntry[] = []
  for (const tab of DOCUMENTO_EDIT_SECTION_TABS) {
    if (includeItens && tab.value === DOCUMENTO_EDIT_ITENS_INSERT_BEFORE) {
      entries.push({ kind: 'itens', value: DOCUMENTO_EDIT_ITENS_TAB, label: itensLabel })
    }
    entries.push({ kind: 'section', ...tab })
  }
  if (includeItens && !entries.some((e) => e.kind === 'itens')) {
    entries.push({ kind: 'itens', value: DOCUMENTO_EDIT_ITENS_TAB, label: itensLabel })
  }
  return entries
}
