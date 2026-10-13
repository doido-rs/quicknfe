import { Download } from 'lucide-react'
import { useState } from 'react'
import { downloadDocumentoXml } from '@/api/documentos'
import { isApiError } from '@/api/client'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'

export function DocumentoXmlDownloadButton({
  documentoId,
}: {
  documentoId: number
  row?: Record<string, unknown>
}) {
  const [pending, setPending] = useState(false)
  const [error, setError] = useState<string | null>(null)
  async function handleDownload() {
    setError(null)
    setPending(true)
    try {
      await downloadDocumentoXml(documentoId)
    } catch (err) {
      setError(isApiError(err) ? err.message : 'Não foi possível baixar o XML.')
    } finally {
      setPending(false)
    }
  }

  return (
    <div className="flex flex-col items-end gap-2">
      <Button
        type="button"
        variant="outline"
        className="gap-2"
        disabled={pending}
        onClick={() => void handleDownload()}
      >
        <Download className="size-4" />
        {pending ? 'Baixando…' : 'Baixar XML'}
      </Button>
      <p className="text-xs text-muted-foreground max-w-xs text-right">
        Rascunho, assinado ou processado — conforme disponível no servidor.
      </p>
      {error && (
        <Alert variant="destructive" className="max-w-sm">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
    </div>
  )
}
