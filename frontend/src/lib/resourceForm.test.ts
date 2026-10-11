import { describe, expect, it } from 'vitest'
import {
  buildCreateBodyWithFiles,
  fieldValue,
  fileToBase64Payload,
  formatFieldValue,
  formatMunicipioRelationLabel,
  rowToForm,
} from './resourceForm'

describe('resourceForm', () => {
  it('maps row to form strings', () => {
    const form = rowToForm({ id: 1, nome: 'X', count: 2 }, [
      { key: 'nome', label: 'Nome' },
      { key: 'count', label: 'Count', type: 'number' },
    ])
    expect(form).toEqual({ nome: 'X', count: '2' })
  })

  it('reads nested field paths', () => {
    expect(fieldValue({ municipio: { uf: 'SP' } }, 'municipio.uf')).toBe('SP')
  })

  it('formats municipio relation from nested row', () => {
    const row = { municipio: { nome: 'São Paulo', uf: 'SP', codigo_ibge: '3550308' } }
    expect(formatMunicipioRelationLabel(row)).toBe('São Paulo (SP · IBGE 3550308)')
    expect(
      formatFieldValue(1, { key: 'municipio_id', relationResource: 'municipios' }, row),
    ).toBe('São Paulo (SP · IBGE 3550308)')
  })

  it('formats empty values', () => {
    expect(formatFieldValue(null)).toBe('—')
    expect(formatFieldValue('ok')).toBe('ok')
  })

  it('embeds file upload as base64 payload keys', async () => {
    const file = new File([new Uint8Array([1, 2, 3])], 'test.pfx', {
      type: 'application/x-pkcs12',
    })
    const b64 = await fileToBase64Payload(file)
    expect(b64.length).toBeGreaterThan(0)

    const body = await buildCreateBodyWithFiles(
      [
        { key: 'tipo', label: 'Tipo', required: true },
        {
          key: 'arquivo',
          label: 'Arquivo',
          type: 'file',
          fileUploadAs: { base64Key: 'arquivo_base64', nameKey: 'arquivo_nome' },
        },
      ],
      { tipo: 'A1' },
      { arquivo: file },
    )
    expect(body).toMatchObject({ tipo: 'A1', arquivo_nome: 'test.pfx' })
    expect(body.arquivo_base64).toBe(b64)
  })
})
