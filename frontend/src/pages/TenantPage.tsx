import { useState } from 'react'
import { Navigate } from 'react-router-dom'
import { isApiError } from '../api/client'
import { useAuth } from '../auth/AuthContext'

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
    <main className="auth-page">
      <h1>Selecione a empresa</h1>
      {isLoading && <p>Carregando…</p>}
      {me && me.empresas.length === 0 && (
        <p>
          Sua conta ainda não está vinculada a nenhuma empresa. Peça ao administrador
          para incluí-lo em <code>empresas_usuarios</code>.
        </p>
      )}
      <ul className="tenant-list">
        {me?.empresas.map(({ empresa, papel }) => (
          <li key={empresa.id}>
            <button type="button" onClick={() => choose(empresa.id)}>
              {empresa.razao_social} ({papel})
            </button>
          </li>
        ))}
      </ul>
      {error && <p className="error">{error}</p>}
      <button type="button" onClick={() => signOut()}>
        Sair
      </button>
    </main>
  )
}
