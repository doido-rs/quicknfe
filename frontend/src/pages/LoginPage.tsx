import { Link, Navigate, useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { isApiError } from '@/api/client'
import { useAuth } from '@/auth/AuthContext'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { AuthPageShell } from '@/components/AuthPageShell'
import { Label } from '@/components/ui/label'

type LoginForm = {
  email: string
  password: string
}

export function LoginPage() {
  const { signIn, me, isLoading } = useAuth()
  const navigate = useNavigate()
  const {
    register,
    handleSubmit,
    formState: { isSubmitting },
  } = useForm<LoginForm>()
  const [error, setError] = useState<string | null>(null)

  if (!isLoading && me) {
    if (!me.company_ativa) return <Navigate to="/tenant" replace />
    if (!me.prestador_ativo) return <Navigate to="/tenant/prestador" replace />
    return <Navigate to="/app" replace />
  }

  async function onSubmit(values: LoginForm) {
    setError(null)
    try {
      await signIn(values.email, values.password)
      navigate({ to: '/tenant' })
    } catch (err) {
      setError(isApiError(err) ? err.message : 'Falha no login')
    }
  }

  return (
    <AuthPageShell>
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>QuickNFe — Entrar</CardTitle>
          <CardDescription>Acesse com seu e-mail e senha.</CardDescription>
        </CardHeader>
        <form onSubmit={handleSubmit(onSubmit)}>
          <CardContent className="flex flex-col gap-4">
            <div className="grid gap-2">
              <Label htmlFor="login-email">E-mail</Label>
              <Input
                id="login-email"
                type="email"
                autoComplete="email"
                {...register('email', { required: true })}
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="login-password">Senha</Label>
              <Input
                id="login-password"
                type="password"
                autoComplete="current-password"
                {...register('password', { required: true })}
              />
            </div>
            {error && (
              <Alert variant="destructive">
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            <Button type="submit" className="w-full" disabled={isSubmitting}>
              Entrar
            </Button>
          </CardContent>
        </form>
        <CardFooter className="justify-center text-sm text-muted-foreground">
          Não tem conta?{' '}
          <Link to="/register" className="text-primary underline-offset-4 hover:underline">
            Cadastre-se
          </Link>
        </CardFooter>
      </Card>
    </AuthPageShell>
  )
}
