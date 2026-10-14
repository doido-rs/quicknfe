import { Link } from '@tanstack/react-router'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  documentoListDate,
  documentoListValor,
  documentoParteListLabel,
  documentoStatusLabel,
} from '@/lib/documentoListFormat'
import { resourceEditTo, resourceShowTo } from '@/lib/resourcePaths'
import type { ResourceDef } from '@/resources/definitions'
import { useDateFormat } from '@/theme/DateFormatContext'

type Row = Record<string, unknown>

export function DocumentoListTable({
  def,
  rows,
  allowDelete,
  onDelete,
}: {
  def: ResourceDef
  rows: Row[]
  allowDelete: boolean
  onDelete: (row: Row) => void
}) {
  const { preference } = useDateFormat()
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Data</TableHead>
          <TableHead>Prestador</TableHead>
          <TableHead>Tomador</TableHead>
          <TableHead>Intermediário</TableHead>
          <TableHead className="text-right">Valor</TableHead>
          <TableHead>Situação</TableHead>
          <TableHead className="w-[1%]">
            <span className="sr-only">Ações</span>
          </TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {rows.length === 0 ? (
          <TableRow>
            <TableCell colSpan={7} className="text-center text-muted-foreground">
              Nenhuma nota fiscal encontrada.
            </TableCell>
          </TableRow>
        ) : (
          rows.map((row) => {
            const id = Number(row.id)
            const status = row.status
            return (
              <TableRow key={String(row.id)}>
                <TableCell className="whitespace-nowrap tabular-nums">
                  {documentoListDate(row, preference)}
                </TableCell>
                <TableCell className="max-w-[12rem] truncate">
                  {documentoParteListLabel(row.prestador_razao_social)}
                </TableCell>
                <TableCell className="max-w-[12rem] truncate">
                  {documentoParteListLabel(row.tomador_razao_social)}
                </TableCell>
                <TableCell className="max-w-[12rem] truncate">
                  {documentoParteListLabel(row.intermediario_razao_social)}
                </TableCell>
                <TableCell className="text-right tabular-nums whitespace-nowrap">
                  {documentoListValor(row)}
                </TableCell>
                <TableCell>
                  <Badge variant="secondary">{documentoStatusLabel(status)}</Badge>
                </TableCell>
                <TableCell>
                  <div className="flex flex-wrap justify-end gap-2">
                    <Button type="button" variant="outline" size="sm" asChild>
                      <Link {...resourceShowTo(def, id)}>Ver</Link>
                    </Button>
                    <Button type="button" variant="outline" size="sm" asChild>
                      <Link {...resourceEditTo(def, id)}>Editar</Link>
                    </Button>
                    {allowDelete && (
                      <Button
                        type="button"
                        variant="destructive"
                        size="sm"
                        onClick={() => onDelete(row)}
                      >
                        Excluir
                      </Button>
                    )}
                  </div>
                </TableCell>
              </TableRow>
            )
          })
        )}
      </TableBody>
    </Table>
  )
}
