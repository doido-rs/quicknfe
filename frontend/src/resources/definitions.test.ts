import { describe, expect, it } from 'vitest'
import { resources, settingsResources, sidebarResources } from './definitions'

describe('resources', () => {
  it('keeps cadastros in settings menu, not main sidebar', () => {
    const sidebarKeys = sidebarResources().map((r) => r.key)
    const settingsKeys = settingsResources().map((r) => r.key)
    expect(sidebarKeys).toContain('documentos')
    expect(sidebarKeys).not.toContain('logs-retorno')
    expect(sidebarKeys).not.toContain('municipios')
    expect(sidebarKeys).not.toContain('prestadores')
    expect(settingsKeys).toEqual(['municipios', 'prestadores', 'clientes', 'intermediarios'])
  })

  it('includes core tenant resources', () => {
    const keys = resources.map((r) => r.key)
    expect(keys).toContain('prestadores')
    expect(keys).toContain('clientes')
    expect(keys).toContain('documentos')
    expect(keys).toContain('intermediarios')
  })

  it('lists notas fiscais first in the menu', () => {
    expect(resources[0]?.key).toBe('documentos')
    expect(resources[0]?.title).toBe('Notas fiscais')
  })

  it('keeps logs out of menus and read-only', () => {
    const logs = resources.find((r) => r.key === 'logs-retorno')
    expect(logs?.allowCreate).toBe(false)
    expect(logs?.menuPlacement).toBe('none')
  })

  it('builds nested itens and logs paths from documento id', () => {
    const documentos = resources.find((r) => r.key === 'documentos')
    const nesteds = Array.isArray(documentos?.nested)
      ? documentos.nested
      : documentos?.nested
        ? [documentos.nested]
        : []
    expect(nesteds[0]?.path(42)).toBe('/api/documentos/42/itens')
    expect(nesteds[1]?.path(42)).toBe('/api/documentos/42/logs-retorno')
    expect(nesteds[1]?.allowCreate).toBe(false)
  })

  it('enables CNC lookup on prestador and tomador', () => {
    expect(resources.find((r) => r.key === 'prestadores')?.cncLookup?.inscricaoFieldKey).toBe(
      'cpf_cnpj',
    )
    expect(resources.find((r) => r.key === 'clientes')?.cncLookup?.inscricaoFieldKey).toBe(
      'cpf_cnpj',
    )
  })

  it('nests certificados under prestador show page', () => {
    const prestadores = resources.find((r) => r.key === 'prestadores')
    const nested = prestadores?.nested
    const certPath =
      nested && !Array.isArray(nested) ? nested.path(7) : undefined
    expect(certPath).toBe('/api/prestadores/7/certificados')
    expect(resources.some((r) => r.key === 'certificados')).toBe(false)
  })

  it('groups nota fiscal form: identificação, prestador|partes, extras', () => {
    const documentos = resources.find((r) => r.key === 'documentos')
    expect(documentos?.formSections).toHaveLength(4)
    expect(documentos?.formSections?.[0]).toMatchObject({
      type: 'identificacao-nota',
      competenciaFieldKey: 'data_competencia',
      serieFieldKey: 'serie',
      numeroFieldKey: 'numero_documento',
      automaticoFieldKey: 'serie_numero_automatico',
    })
    expect(documentos?.formSections?.[1]).toMatchObject({
      type: 'prestador-partes-row',
      prestadorFieldKey: 'empresa_id',
      partesLocalidade: true,
    })
    expect(documentos?.fields.some((f) => f.key === 'tomador_localidade')).toBe(true)
    expect(documentos?.fields.some((f) => f.key === 'tomador_exterior_pais')).toBe(true)
    expect(documentos?.fields.some((f) => f.key === 'data_competencia')).toBe(true)
    expect(documentos?.formSections?.[2]).toMatchObject({
      type: 'municipios-nota',
      emissaoFieldKey: 'municipio_emissao_id',
      prestacaoFieldKey: 'municipio_prestacao_id',
    })
    expect(documentos?.formSections?.[3]).toMatchObject({
      type: 'fields',
      title: 'Dados extras',
      fieldKeys: ['modelo', 'status', 'lote_id'],
      columns: 1,
    })
  })
})
