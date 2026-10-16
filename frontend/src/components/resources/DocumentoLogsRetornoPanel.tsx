import { useQuery } from '@tanstack/react-query'
import { useMemo } from 'react'
import { resourceApi } from '@/api/client'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { formatDateForDisplay } from '@/lib/dateFormat'
import { useDateFormat } from '@/theme/DateFormatContext'

function logCreatedAt(row: Record<string, unknown>): unknown {
  return row.data_registro ?? row.created_at
}

function compareLogRows(a: Record<string, unknown>, b: Record<string, unknown>): number {
  const ta = String(logCreatedAt(a) ?? '')
  const tb = String(logCreatedAt(b) ?? '')
  return tb.localeCompare(ta)
}

export function DocumentoLogsRetornoPanel({ documentoId }: { documentoId: number }) {
  const path = `/api/documentos/${documentoId}/logs-retorno`
  const dateFormat = useDateFormat()
  const api = resourceApi<Record<string, unknown>>(path)

  const { data: page, isLoading } = useQuery({
    queryKey: [path, 'list'],
    queryFn: () => api.listPage({ pageSize: 50 }),
  })

  const rows = useMemo(() => {
    const items = page?.items ?? []
    return [...items].sort(compareLogRows)
  }, [page?.items])

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Logs de retorno (SEFAZ)</CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <p className="text-sm text-muted-foreground">Carregando logs…</p>
        ) : rows.length === 0 ? (
          <p className="text-sm text-muted-foreground">Nenhum registro de retorno da SEFIN.</p>
        ) : (
          <div className="w-full max-w-full [&_[data-slot=table-container]]:overflow-x-hidden">
            <Table className="table-fixed">
              <TableHeader>
                <TableRow>
                  <TableHead className="w-[5.5rem]">Código</TableHead>
                  <TableHead className="min-w-0">Mensagem</TableHead>
                  <TableHead className="w-[10.5rem]">Registrado em</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {rows.map((row) => {
                const codigo =
                  row.codigo_erro != null && String(row.codigo_erro).trim() !== ''
                    ? String(row.codigo_erro)
                    : '—'
                const mensagem =
                  row.mensagem_retorno != null && String(row.mensagem_retorno).trim() !== ''
                    ? String(row.mensagem_retorno)
                    : '—'
                const correcao =
                  row.correcao_sugerida != null && String(row.correcao_sugerida).trim() !== ''
                    ? String(row.correcao_sugerida)
                    : null
                const registradoEm = formatDateForDisplay(logCreatedAt(row), dateFormat)

                  return (
                    <TableRow key={String(row.id)}>
                      <TableCell className="align-top font-medium whitespace-normal">
                        {codigo}
                      </TableCell>
                      <TableCell className="max-w-0 align-top whitespace-normal break-words [overflow-wrap:anywhere]">
                        <p className="leading-snug">{mensagem}</p>
                        {correcao ? (
                          <p className="mt-1 text-xs leading-snug text-muted-foreground break-words [overflow-wrap:anywhere]">
                            {correcao}
                          </p>
                        ) : null}
                      </TableCell>
                      <TableCell className="align-top whitespace-normal text-muted-foreground text-xs sm:text-sm">
                        {registradoEm}
                      </TableCell>
                    </TableRow>
                  )
                })}
              </TableBody>
            </Table>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
