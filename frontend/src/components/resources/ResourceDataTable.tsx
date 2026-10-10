import { Link } from '@tanstack/react-router'
import {
  createColumnHelper,
  flexRender,
  tableFeatures,
  useTable,
} from '@tanstack/react-table'
import { useMemo } from 'react'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { rowLabel } from '@/lib/resourceForm'
import { resourceEditTo, resourceShowTo } from '@/lib/resourcePaths'
import type { ResourceDef } from '@/resources/definitions'

type Row = Record<string, unknown>

const features = tableFeatures({})

export function ResourceDataTable({
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
  const columns = useMemo(() => {
    const helper = createColumnHelper<typeof features, Row>()
    return helper.columns([
      helper.accessor('id', {
        header: 'ID',
        cell: (info) => String(info.getValue() ?? ''),
      }),
      helper.display({
        id: 'summary',
        header: 'Resumo',
        cell: ({ row }) => rowLabel(row.original, def.fields),
      }),
      helper.display({
        id: 'actions',
        header: () => <span className="sr-only">Ações</span>,
        cell: ({ row }) => {
          const id = Number(row.original.id)
          return (
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
                  onClick={() => onDelete(row.original)}
                >
                  Excluir
                </Button>
              )}
            </div>
          )
        },
      }),
    ])
  }, [def, allowDelete, onDelete])

  const table = useTable({ features, data: rows, columns })

  return (
    <Table>
      <TableHeader>
        {table.getHeaderGroups().map((headerGroup) => (
          <TableRow key={headerGroup.id}>
            {headerGroup.headers.map((header) => (
              <TableHead
                key={header.id}
                className={header.column.id === 'actions' ? 'text-right' : undefined}
              >
                {header.isPlaceholder
                  ? null
                  : flexRender(header.column.columnDef.header, header.getContext())}
              </TableHead>
            ))}
          </TableRow>
        ))}
      </TableHeader>
      <TableBody>
        {table.getRowModel().rows.length === 0 ? (
          <TableRow>
            <TableCell colSpan={columns.length} className="text-muted-foreground">
              Nenhum registro.
            </TableCell>
          </TableRow>
        ) : (
          table.getRowModel().rows.map((row) => (
            <TableRow key={row.id}>
              {row.getAllCells().map((cell) => (
                <TableCell
                  key={cell.id}
                  className={cell.column.id === 'actions' ? 'text-right' : undefined}
                >
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </TableCell>
              ))}
            </TableRow>
          ))
        )}
      </TableBody>
    </Table>
  )
}
