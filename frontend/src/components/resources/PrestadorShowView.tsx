import { Link } from '@tanstack/react-router'
import { NestedResourcePanel } from '@/components/resources/NestedResourcePanel'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { fieldValue, formatFieldValue, rowLabel } from '@/lib/resourceForm'
import { resourceEditTo, resourceListTo } from '@/lib/resourcePaths'
import { resourceNesteds, type ResourceDef } from '@/resources/definitions'

export function PrestadorShowView({
  def,
  recordId,
  row,
  allowDelete,
  onRequestDelete,
}: {
  def: ResourceDef
  recordId: number
  row: Record<string, unknown>
  allowDelete: boolean
  onRequestDelete: () => void
}) {
  const nesteds = resourceNesteds(def)
  const certNested = nesteds.find((n) => n.path(recordId).includes('/certificados'))

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <Button variant="ghost" size="sm" asChild className="mb-2 -ml-2">
            <Link {...resourceListTo(def)}>← Voltar à lista</Link>
          </Button>
          <p className="text-sm text-muted-foreground">
            ID {String(row.id)} · {rowLabel(row, def.fields)}
          </p>
        </div>
        <div className="flex flex-wrap items-start justify-end gap-2">
          <Button variant="outline" asChild>
            <Link {...resourceEditTo(def, recordId)}>Editar</Link>
          </Button>
          {allowDelete ? (
            <Button variant="destructive" onClick={onRequestDelete}>
              Excluir
            </Button>
          ) : null}
        </div>
      </div>

      <Tabs defaultValue="dados">
        <TabsList aria-label="Seções do prestador">
          <TabsTrigger value="dados">Dados do prestador</TabsTrigger>
          <TabsTrigger value="certificados">Certificados digitais</TabsTrigger>
        </TabsList>

        <TabsContent value="dados">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Dados do prestador</CardTitle>
            </CardHeader>
            <CardContent>
              <dl className="grid gap-3 sm:grid-cols-2">
                {def.fields
                  .filter((field) => !field.formOnly)
                  .map((field) => (
                    <div key={field.key}>
                      <dt className="text-sm font-medium text-muted-foreground">{field.label}</dt>
                      <dd className="text-sm">
                        {formatFieldValue(fieldValue(row, field.key), field, row)}
                      </dd>
                    </div>
                  ))}
              </dl>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="certificados">
          {certNested ? (
            <NestedResourcePanel parentId={recordId} nested={certNested} />
          ) : null}
        </TabsContent>
      </Tabs>
    </div>
  )
}
