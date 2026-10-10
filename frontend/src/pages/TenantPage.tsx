import { useState } from 'react'
import { Navigate } from '@tanstack/react-router'
import { isApiError } from '@/api/client'
import { useAuth } from '@/auth/AuthContext'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { AuthPageShell } from '@/components/AuthPageShell'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

export function TenantPage() {
  const { me, isLoading, selectCompany, signOut } = useAuth()
  const [error, setError] = useState<string | null>(null)

  if (!isLoading && !me) {
    return <Navigate to="/login" replace />
  }

  if (!isLoading && me?.company_ativa) {
    return <Navigate to="/tenant/prestador" replace />
  }

  async function choose(companyId: number) {
    setError(null)
    try {
      await selectCompany(companyId)
    } catch (err) {
      setError(isApiError(err) ? err.message : 'Não foi possível selecionar a company')
    }
  }

  return (
    <AuthPageShell>
      <Card className="w-full max-w-lg">
        <CardHeader>
          <CardTitle>Selecione a company</CardTitle>
          <CardDescription>Escolha o tenant (organização) para esta sessão.</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          {isLoading && <p className="text-sm text-muted-foreground">Carregando…</p>}
          {me && me.companies.length === 0 && (
            <p className="text-sm text-muted-foreground">
              Sua conta ainda não está vinculada a nenhuma company. Peça ao administrador para
              incluí-lo em <code className="rounded bg-muted px-1">companies_usuarios</code>.
            </p>
          )}
          <div className="flex flex-col gap-2">
            {me?.companies.map(({ company, papel }) => (
              <Button
                key={company.id}
                type="button"
                variant="outline"
                className="h-auto justify-start py-3 text-left"
                onClick={() => choose(company.id)}
              >
                {company.nome} ({papel})
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
