import { useState } from 'react'
import { Navigate } from 'react-router-dom'
import { isApiError } from '@/api/client'
import { useAuth } from '@/auth/AuthContext'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { AuthPageShell } from '@/components/AuthPageShell'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

export function TenantPage() {
  const { me, isLoading, selectEmpresa, signOut } = useAuth()
  const [error, setError] = useState<string | null>(null)

  if (!isLoading && !me) {
    return <Navigate to="/login" replace />
  }

  if (!isLoading && me?.empresa_ativa) {
    return <Navigate to="/app" replace />
  }

  async function choose(empresaId: number) {
    setError(null)
    try {
      await selectEmpresa(empresaId)
    } catch (err) {
      setError(isApiError(err) ? err.message : 'Não foi possível selecionar a empresa')
    }
  }

  return (
    <AuthPageShell>
      <Card className="w-full max-w-lg">
        <CardHeader>
          <CardTitle>Selecione a empresa</CardTitle>
          <CardDescription>Escolha o tenant ativo para esta sessão.</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          {isLoading && <p className="text-sm text-muted-foreground">Carregando…</p>}
          {me && me.empresas.length === 0 && (
            <p className="text-sm text-muted-foreground">
              Sua conta ainda não está vinculada a nenhuma empresa. Peça ao administrador
              para incluí-lo em <code className="rounded bg-muted px-1">empresas_usuarios</code>.
            </p>
          )}
          <div className="flex flex-col gap-2">
            {me?.empresas.map(({ empresa, papel }) => (
              <Button
                key={empresa.id}
                type="button"
                variant="outline"
                className="h-auto justify-start py-3 text-left"
                onClick={() => choose(empresa.id)}
              >
                {empresa.razao_social} ({papel})
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
