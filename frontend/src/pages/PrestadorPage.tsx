import { useState } from 'react'
import { Navigate } from 'react-router-dom'
import { isApiError } from '@/api/client'
import { useAuth } from '@/auth/AuthContext'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { AuthPageShell } from '@/components/AuthPageShell'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

export function PrestadorPage() {
  const { me, isLoading, selectPrestador, signOut } = useAuth()
  const [error, setError] = useState<string | null>(null)

  if (!isLoading && !me) {
    return <Navigate to="/login" replace />
  }

  if (!isLoading && me && !me.company_ativa) {
    return <Navigate to="/tenant" replace />
  }

  if (!isLoading && me?.prestador_ativo) {
    return <Navigate to="/app" replace />
  }

  async function choose(empresaId: number) {
    setError(null)
    try {
      await selectPrestador(empresaId)
    } catch (err) {
      setError(isApiError(err) ? err.message : 'Não foi possível selecionar o prestador')
    }
  }

  return (
    <AuthPageShell>
      <Card className="w-full max-w-lg">
        <CardHeader>
          <CardTitle>Selecione o prestador</CardTitle>
          <CardDescription>
            Company: <strong>{me?.company_ativa?.nome}</strong>. Escolha o prestador de serviços
            ativo.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          {isLoading && <p className="text-sm text-muted-foreground">Carregando…</p>}
          {me && me.prestadores.length === 0 && (
            <p className="text-sm text-muted-foreground">
              Nenhum prestador cadastrado nesta company. Cadastre em{' '}
              <code className="rounded bg-muted px-1">/api/prestadores</code> ou peça ao
              administrador.
            </p>
          )}
          <div className="flex flex-col gap-2">
            {me?.prestadores.map((p) => (
              <Button
                key={p.id}
                type="button"
                variant="outline"
                className="h-auto justify-start py-3 text-left"
                onClick={() => choose(p.id)}
              >
                {p.razao_social} — {p.cpf_cnpj}
              </Button>
            ))}
          </div>
          {error && (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
          <Button type="button" variant="ghost" onClick={() => signOut()}>
            Sair
          </Button>
        </CardContent>
      </Card>
    </AuthPageShell>
  )
}
