import { afterEach, describe, expect, it, vi } from 'vitest'
import * as client from './client'
import { consultaCncCad } from './cnc'

afterEach(() => {
  vi.restoreAllMocks()
})

describe('consultaCncCad', () => {
  it('builds query with digits-only inscricao and optional params', async () => {
    vi.spyOn(client, 'apiFetch').mockResolvedValue({ itens: [], alertas: [], erros: [] })
    await consultaCncCad({
      inscricaoFederal: '12.345.678/0001-90',
      codMunicipio: 3304557,
      indicadorMunicipal: '  IM123 ',
      empresaId: 7,
    })
    const url = (client.apiFetch as ReturnType<typeof vi.fn>).mock.calls[0][0] as string
    expect(url).toContain('inscricao_federal=12345678000190')
    expect(url).toContain('cod_municipio=3304557')
    expect(url).toContain('indicador_municipal=IM123')
    expect(url).toContain('empresa_id=7')
  })
})
