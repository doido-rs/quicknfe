import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { resourceApi } from '@/api/client'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { buildBody, emptyForm, rowLabel, useResourceFieldForm } from '@/lib/resourceForm'
import type { ResourceDef } from '@/resources/definitions'

export function NestedResourcePanel({
  parentId,
  nested,
}: {
  parentId: number
  nested: NonNullable<ResourceDef['nested']>
}) {
  const path = nested.path(parentId)
  const api = resourceApi<Record<string, unknown>>(path)
  const queryClient = useQueryClient()
  const form = useResourceFieldForm(nested.fields)
  const { register, handleSubmit, reset } = form

  const { data: rows = [] } = useQuery({
    queryKey: [path, 'list'],
    queryFn: () => api.list(),
  })

  const createMutation = useMutation({
    mutationFn: (values: Record<string, string>) => api.create(buildBody(nested.fields, values)),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [path] })
      reset(emptyForm(nested.fields))
    },
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{nested.label}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <form
          className="grid gap-4 md:grid-cols-2"
          onSubmit={handleSubmit((values) => createMutation.mutate(values))}
        >
          {nested.fields.map((f) => (
            <div key={f.key} className="grid gap-2">
              <Label htmlFor={`nested-${f.key}`}>{f.label}</Label>
              <Input
                id={`nested-${f.key}`}
                type={f.type === 'number' ? 'number' : 'text'}
                {...register(f.key, { required: f.required })}
              />
            </div>
          ))}
          <div className="md:col-span-2">
            <Button type="submit" disabled={createMutation.isPending}>
              Adicionar item
            </Button>
          </div>
        </form>
        <ul className="list-disc space-y-1 pl-5 text-sm">
          {rows.map((r) => (
            <li key={String(r.id)}>{rowLabel(r, nested.fields)}</li>
          ))}
        </ul>
      </CardContent>
    </Card>
  )
}
