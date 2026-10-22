import { describe, expect, it } from 'vitest'
import {
  buildBody,
  buildCreateBodyWithFiles,
  buildNestedCreateBody,
  fieldValue,
  fileToBase64Payload,
  formatFieldValue,
  formatMunicipioRelationLabel,
  rowToForm,
} from './resourceForm'

describe('resourceForm', () => {
  it('buildNestedCreateBody copies descricao from descricao_servico', () => {
    const body = buildNestedCreateBody(
      {
        fields: [
          { key: 'descricao_servico', label: 'Serviço' },
          { key: 'numero_item', label: 'N', type: 'number' },
        ],
        prepareCreateBody: (b) => {
          const servico = b.descricao_servico
          if (
            (b.descricao == null || String(b.descricao).trim() === '') &&
            servico != null &&
            String(servico).trim() !== ''
          ) {
            b.descricao = String(servico).trim().slice(0, 500)
          }
          return b
        },
      },
      { descricao_servico: 'Consultoria', numero_item: '1' },
    )
    expect(body.descricao).toBe('Consultoria')
    expect(body.numero_item).toBe(1)
  })

  it('maps row to form strings', () => {
    const form = rowToForm({ id: 1, nome: 'X', count: 2, total: 10.5 }, [
      { key: 'nome', label: 'Nome' },
      { key: 'count', label: 'Count', type: 'number' },
      { key: 'total', label: 'Total', type: 'money' },
    ])
    expect(form).toEqual({ nome: 'X', count: '2', total: '10.50' })
  })

  it('buildBody sends money as rounded number', () => {
    const body = buildBody([{ key: 'valor', label: 'V', type: 'money' }], {
      valor: '123.456',
    })
    expect(body.valor).toBe(123.46)
  })

  it('maps and sends percent fields', () => {
    const form = rowToForm({ aliq: 2.5 }, [{ key: 'aliq', label: 'Alíquota', type: 'percent' }])
    expect(form.aliq).toBe('2.5000')
    const body = buildBody([{ key: 'aliq', label: 'Alíquota', type: 'percent' }], {
      aliq: '5.12345',
    })
    expect(body.aliq).toBe(5.1235)
  })

  it('reads nested field paths', () => {
    expect(fieldValue({ municipio: { uf: 'SP' } }, 'municipio.uf')).toBe('SP')
  })

  it('formats municipio relation from nested row', () => {
    const row = { municipio: { nome: 'São Paulo', uf: 'SP', codigo_ibge: '3550308' } }
    expect(formatMunicipioRelationLabel(row)).toBe('São Paulo (SP · IBGE 3550308)')
    expect(
      formatFieldValue(1, { key: 'municipio_id', label: 'Município', relationResource: 'municipios' }, row),
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

  it('nested certificado create body omits data_validade', () => {
    const body = buildNestedCreateBody(
      {
        fields: [
          { key: 'tipo', label: 'Tipo', type: 'enum', required: true },
          { key: 'senha', label: 'Senha', required: true },
        ],
      },
      { tipo: 'A1', senha: 'secret' },
    )
    expect(body).toEqual({ tipo: 'A1', senha: 'secret' })
    expect(body).not.toHaveProperty('data_validade')
  })
})
