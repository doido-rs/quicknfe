import { Navigate, NavLink, Outlet } from 'react-router-dom'
import { useAuth } from '../auth/AuthContext'
import { resources } from '../resources/definitions'

export function AppLayout() {
  const { me, isLoading, signOut } = useAuth()

  if (!isLoading && !me) {
    return <Navigate to="/login" replace />
  }

  if (!isLoading && me && !me.empresa_ativa) {
    return <Navigate to="/tenant" replace />
  }

  return (
    <div className="app-shell">
      <header>
        <strong>QuickNFe</strong>
        {me?.empresa_ativa && (
          <span className="tenant-badge">{me.empresa_ativa.razao_social}</span>
        )}
        <button type="button" onClick={() => signOut()}>
          Sair
        </button>
      </header>
      <nav>
        {resources.map((r) => (
          <NavLink key={r.key} to={`/app/${r.key}`}>
            {r.title}
          </NavLink>
        ))}
      </nav>
      <main>
        <Outlet />
      </main>
    </div>
  )
}
