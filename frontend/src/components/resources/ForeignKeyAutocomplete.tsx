import { useQuery } from '@tanstack/react-query'
import { useEffect, useId, useMemo, useState } from 'react'
import { resourceApi } from '@/api/client'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { relationApiPath, relationOptionLabel } from '@/lib/relations'
import type { FieldDef } from '@/resources/definitions'

function useDebouncedValue<T>(value: T, delayMs: number): T {
  const [debounced, setDebounced] = useState(value)
  useEffect(() => {
    const t = window.setTimeout(() => setDebounced(value), delayMs)
    return () => window.clearTimeout(t)
  }, [value, delayMs])
  return debounced
}

export function ForeignKeyAutocomplete({
  field,
  resourceKey,
  value,
  onChange,
  onSelectRow,
  required,
}: {
  field: FieldDef
  resourceKey: string
  value: string
  onChange: (next: string) => void
  onSelectRow?: (row: Record<string, unknown>) => void
  required?: boolean
}) {
  const listId = useId()
  const path = relationApiPath(resourceKey)
  const api = useMemo(
    () => (path ? resourceApi<Record<string, unknown>>(path) : null),
    [path],
  )

  const selectedId = value.trim() === '' ? null : Number(value)
  const [search, setSearch] = useState('')
  const [open, setOpen] = useState(false)
  const [pageToken, setPageToken] = useState<string | undefined>()
  const debouncedSearch = useDebouncedValue(search, 250)

  const { data: selectedRow } = useQuery({
    queryKey: [resourceKey, 'option', selectedId],
    queryFn: () => api!.get(selectedId!),
    enabled: api != null && selectedId != null && Number.isFinite(selectedId),
  })

  const { data: page, isFetching } = useQuery({
    queryKey: [resourceKey, 'search', debouncedSearch, pageToken ?? 'start'],
    queryFn: () =>
      api!.listPage({
        q: debouncedSearch || undefined,
        pageToken,
        pageSize: 15,
      }),
    enabled: api != null && open,
  })

  useEffect(() => {
    setPageToken(undefined)
  }, [debouncedSearch])

  const options = page?.items ?? []
  const nextToken = page?.next_page_token

  const displayValue =
    open || search
      ? search
      : selectedRow
        ? relationOptionLabel(resourceKey, selectedRow)
        : value

  if (!api) {
    return (
      <Input
        id={`field-${field.key}`}
        type="number"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        required={required}
      />
    )
  }

  return (
    <div className="relative grid gap-2">
      <Label htmlFor={listId}>{field.label}</Label>
      <Input
        id={listId}
        role="combobox"
        aria-expanded={open}
        aria-autocomplete="list"
        autoComplete="off"
        value={displayValue}
        required={required && (value === '' || value == null)}
        onFocus={() => {
          setOpen(true)
          if (!search && selectedRow) setSearch('')
        }}
        onBlur={() => {
          window.setTimeout(() => setOpen(false), 150)
        }}
        onChange={(e) => {
          setSearch(e.target.value)
          setOpen(true)
          if (e.target.value === '') onChange('')
        }}
      />
      {open && (
        <div className="absolute top-full z-20 mt-1 max-h-56 w-full overflow-auto rounded-md border bg-popover p-1 shadow-md">
          {isFetching && options.length === 0 && (
            <p className="px-2 py-1.5 text-xs text-muted-foreground">Buscando…</p>
          )}
          {!isFetching && options.length === 0 && (
            <p className="px-2 py-1.5 text-xs text-muted-foreground">Nenhum resultado.</p>
          )}
          {options.map((row) => {
            const id = Number(row.id)
            const label = relationOptionLabel(resourceKey, row)
            return (
              <button
                key={String(row.id)}
                type="button"
                className="flex w-full rounded-sm px-2 py-1.5 text-left text-sm hover:bg-accent"
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => {
                  onChange(String(id))
                  onSelectRow?.(row)
                  setSearch(label)
                  setOpen(false)
                }}
              >
                {label}
              </button>
            )
          })}
          {nextToken && (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="mt-1 w-full"
              disabled={isFetching}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => setPageToken(nextToken)}
            >
              Carregar mais
            </Button>
          )}
        </div>
      )}
      {value && (
        <Button
          type="button"
          variant="link"
          className="h-auto justify-start px-0 text-xs text-muted-foreground"
          onClick={() => {
            onChange('')
            setSearch('')
          }}
        >
          Limpar seleção
        </Button>
      )}
    </div>
  )
}
