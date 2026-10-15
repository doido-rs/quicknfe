import { Link } from '@tanstack/react-router'
import { useEffect, useMemo } from 'react'
import { DocumentoAssinarButton } from '@/components/resources/DocumentoAssinarButton'
import { DocumentoTransmitirButton } from '@/components/resources/DocumentoTransmitirButton'
import { DocumentoXmlDownloadButton } from '@/components/resources/DocumentoXmlDownloadButton'
import { NestedResourcePanel } from '@/components/resources/NestedResourcePanel'
import { ResourceFormFields } from '@/components/resources/ResourceFormFields'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { flattenDocumentoRowToForm } from '@/lib/documentoParteLocalidade'
import { documentoStatusLabel } from '@/lib/documentoListFormat'
import { buildDocumentoRelationLabels } from '@/lib/documentoShowLabels'
import { rowToForm, useResourceFieldForm } from '@/lib/resourceForm'
import { resourceEditTo, resourceListTo } from '@/lib/resourcePaths'
import { useDocumentoProcessamentoPoll } from '@/hooks/useDocumentoProcessamentoPoll'
import { resourceNesteds, type ResourceDef } from '@/resources/definitions'

export function DocumentoShowView({
  def,
  recordId,
  row,
}: {
  def: ResourceDef
  recordId: number
  row: Record<string, unknown>
}) {
  const form = useResourceFieldForm(def.fields)
  const relationLabels = useMemo(() => buildDocumentoRelationLabels(row), [row])
  useDocumentoProcessamentoPoll(recordId, row)

  useEffect(() => {
    const base = rowToForm(row, def.fields)
    form.reset(flattenDocumentoRowToForm(row, base))
  }, [row, def.fields, form])

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <Button variant="ghost" size="sm" asChild className="mb-2 -ml-2">
            <Link {...resourceListTo(def)}>← Voltar à lista</Link>
          </Button>
          <div className="flex flex-wrap items-center gap-2">
            <p className="text-sm text-muted-foreground">
              ID {String(row.id)}
              {row.serie != null && row.numero_documento != null
                ? ` · Série ${String(row.serie)} · Nº ${String(row.numero_documento)}`
                : ''}
            </p>
            <Badge variant="secondary">{documentoStatusLabel(row.status)}</Badge>
          </div>
        </div>
        <div className="flex flex-wrap items-start justify-end gap-2">
          <DocumentoAssinarButton documentoId={recordId} row={row} />
          <DocumentoTransmitirButton documentoId={recordId} row={row} />
          <DocumentoXmlDownloadButton documentoId={recordId} row={row} />
          <Button variant="outline" asChild>
            <Link {...resourceEditTo(def, recordId)}>Editar</Link>
          </Button>
        </div>
      </div>

      <ResourceFormFields
        fields={def.fields}
        formSections={def.formSections}
        form={form}
        readOnly
        relationDisplayLabels={relationLabels}
        displayRow={row}
      />

      {resourceNesteds(def).map((nested) => (
        <NestedResourcePanel
          key={nested.path(recordId)}
          parentId={recordId}
          nested={nested}
        />
      ))}
    </div>
  )
}
