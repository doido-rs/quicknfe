import { Link, useNavigate } from '@tanstack/react-router'
import { useForm } from 'react-hook-form'
import { useState } from 'react'
import { isApiError } from '@/api/client'
import { useAuth } from '@/auth/AuthContext'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { AuthPageShell } from '@/components/AuthPageShell'
import { Label } from '@/components/ui/label'

type RegisterForm = {
  email: string
  password: string
}

export function RegisterPage() {
  const { signUp } = useAuth()
  const navigate = useNavigate()
  const {
    register,
    handleSubmit,
    formState: { isSubmitting },
  } = useForm<RegisterForm>()
  const [error, setError] = useState<string | null>(null)

  async function onSubmit(values: RegisterForm) {
    setError(null)
    try {
      await signUp(values.email, values.password)
      navigate({ to: '/tenant' })
    } catch (err) {
      setError(isApiError(err) ? err.message : 'Falha no cadastro')
    }
  }

  return (
    <AuthPageShell>
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Criar conta</CardTitle>
          <CardDescription>Cadastre-se para acessar o QuickNFe.</CardDescription>
        </CardHeader>
        <form onSubmit={handleSubmit(onSubmit)}>
          <CardContent className="flex flex-col gap-4">
            <div className="grid gap-2">
              <Label htmlFor="register-email">E-mail</Label>
              <Input
                id="register-email"
                type="email"
                autoComplete="email"
                {...register('email', { required: true })}
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="register-password">Senha</Label>
              <Input
                id="register-password"
                type="password"
                autoComplete="new-password"
                minLength={8}
                {...register('password', { required: true, minLength: 8 })}
              />
            </div>
            {error && (
              <Alert variant="destructive">
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            <Button type="submit" className="w-full" disabled={isSubmitting}>
              Cadastrar
            </Button>
          </CardContent>
        </form>
        <CardFooter className="justify-center text-sm text-muted-foreground">
          Já tem conta?{' '}
          <Link to="/login" className="text-primary underline-offset-4 hover:underline">
            Entrar
          </Link>
        </CardFooter>
      </Card>
    </AuthPageShell>
  )
}
