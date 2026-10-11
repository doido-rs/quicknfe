import { useMutation } from '@tanstack/react-query'
import { useState } from 'react'
import type { UseFormReturn } from 'react-hook-form'
import { useAuth } from '@/auth/AuthContext'
import { consultaCncCad, type CncCadastroItem } from '@/api/cnc'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { applyCncItemToForm, onlyDigits } from '@/lib/cncConsulta'
import { resourceApi } from '@/api/client'
import type { CncLookupDef } from '@/resources/definitions'

async function codigoIbgeFromMunicipioId(municipioId: string): Promise<number | undefined> {
  const id = Number(municipioId)
  if (!Number.isFinite(id)) return undefined
  const row = await resourceApi<{ codigo_ibge?: string }>('/api/municipios').get(id)
  const cod = row.codigo_ibge?.replace(/\D/g, '')
  if (!cod) return undefined
  return Number(cod)
}

export function CncConsultaField({
  lookup,
  form,
}: {
  lookup: CncLookupDef
  form: UseFormReturn<Record<string, string>>
}) {
  const { me } = useAuth()
  const inscricao = form.watch(lookup.inscricaoFieldKey) ?? ''
  const municipioId = lookup.municipioFieldKey
    ? (form.watch(lookup.municipioFieldKey) ?? '')
    : ''
  const [message, setMessage] = useState<string | null>(null)
  const [choices, setChoices] = useState<CncCadastroItem[] | null>(null)

  const mutation = useMutation({
    mutationFn: async () => {
      const digits = onlyDigits(inscricao)
      if (digits.length !== 11 && digits.length !== 14) {
        throw new Error('Informe CPF (11) ou CNPJ (14 dígitos) antes de consultar o CNC.')
      }
      let codMunicipio: number | undefined
      if (municipioId.trim()) {
        codMunicipio = await codigoIbgeFromMunicipioId(municipioId)
      }
      return consultaCncCad({
        inscricaoFederal: digits,
        codMunicipio,
        empresaId: me?.prestador_ativo?.id,
      })
    },
    onSuccess: (data) => {
      setMessage(null)
      if (data.erros?.length) {
        const txt = data.erros.map((e) => e.descricao ?? e.codigo).filter(Boolean).join('; ')
        setMessage(txt || 'Consulta CNC rejeitada.')
        setChoices(null)
        return
      }
      if (!data.itens?.length) {
        setMessage(
          data.status_processamento === 'NENHUM_DOCUMENTO_LOCALIZADO'
            ? 'Nenhum cadastro localizado no CNC para este documento.'
            : 'Nenhum cadastro retornado.',
        )
        setChoices(null)
        return
      }
      if (data.itens.length === 1) {
        applyCncItemToForm(form, lookup, data.itens[0]!)
        setMessage('Dados preenchidos a partir do CNC.')
        setChoices(null)
        return
      }
      setChoices(data.itens)
      setMessage('Mais de um vínculo municipal encontrado — escolha um registro.')
    },
    onError: (err) => {
      setChoices(null)
      setMessage(err instanceof Error ? err.message : 'Falha na consulta CNC.')
    },
  })

  return (
    <div className="md:col-span-2 rounded-lg border border-dashed bg-muted/30 p-4">
      <div className="flex flex-wrap items-end gap-3">
        <div className="min-w-[12rem] flex-1">
          <Label className="text-muted-foreground">Consulta CNC (Receita / cadastro nacional)</Label>
          <p className="mt-1 text-sm font-medium tabular-nums">
            {onlyDigits(inscricao) || '— informe o CPF/CNPJ no formulário'}
          </p>
        </div>
        <Button
          type="button"
          variant="secondary"
          disabled={mutation.isPending}
          onClick={() => mutation.mutate()}
        >
          {mutation.isPending ? 'Consultando…' : 'Consultar CNC'}
        </Button>
      </div>
      {message && <p className="mt-2 text-sm text-muted-foreground">{message}</p>}
      {choices && choices.length > 1 && (
        <ul className="mt-3 space-y-2">
          {choices.map((item, idx) => (
            <li key={`${item.inscricao_municipal ?? idx}`}>
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="h-auto w-full justify-start whitespace-normal py-2 text-left"
                onClick={() => {
                  applyCncItemToForm(form, lookup, item)
                  setChoices(null)
                  setMessage('Dados preenchidos a partir do CNC.')
                }}
              >
                {item.razao_social ?? item.inscricao} — IM {item.inscricao_municipal ?? '—'}
                {item.codigo_municipio_ibge ? ` · IBGE ${item.codigo_municipio_ibge}` : ''}
              </Button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
