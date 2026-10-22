import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { documentoReadinessIssues } from '@/lib/documentoReadiness'

export function DocumentoReadinessAlert({
  row,
  itensCount,
}: {
  row: Record<string, unknown>
  itensCount?: number
}) {
  const issues = documentoReadinessIssues(row, itensCount)
  if (issues.length === 0) return null

  return (
    <Alert variant="default" className="border-amber-500/50 bg-amber-500/5">
      <AlertTitle>Antes de assinar</AlertTitle>
      <AlertDescription>
        <ul className="list-inside list-disc space-y-1">
          {issues.map((issue) => (
            <li key={issue.id}>{issue.message}</li>
          ))}
        </ul>
      </AlertDescription>
    </Alert>
  )
}
