import { useState, type FormEvent } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { isApiError } from '../api/client'
import { useAuth } from '../auth/AuthContext'

export function RegisterPage() {
  const { signUp } = useAuth()
  const navigate = useNavigate()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)

  async function onSubmit(e: FormEvent) {
    e.preventDefault()
    setError(null)
    try {
      await signUp(email, password)
      navigate('/tenant')
    } catch (err) {
      setError(isApiError(err) ? err.message : 'Falha no cadastro')
    }
  }

  return (
    <main className="auth-page">
      <h1>Criar conta</h1>
      <form onSubmit={onSubmit}>
        <label>
          E-mail
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            required
          />
        </label>
        <label>
          Senha
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
            minLength={8}
          />
        </label>
        {error && <p className="error">{error}</p>}
        <button type="submit">Cadastrar</button>
      </form>
      <p>
        Já tem conta? <Link to="/login">Entrar</Link>
      </p>
    </main>
  )
}
