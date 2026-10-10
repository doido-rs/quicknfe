import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { AppLayout } from './pages/AppLayout'
import { LoginPage } from './pages/LoginPage'
import { RegisterPage } from './pages/RegisterPage'
import { ResourceLayout } from './pages/ResourceLayout'
import { PrestadorPage } from './pages/PrestadorPage'
import { TenantPage } from './pages/TenantPage'
import { ResourceEditPage } from './pages/resources/ResourceEditPage'
import { ResourceListPage } from './pages/resources/ResourceListPage'
import { ResourceNewPage } from './pages/resources/ResourceNewPage'
import { ResourceShowPage } from './pages/resources/ResourceShowPage'

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Navigate to="/login" replace />} />
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/tenant" element={<TenantPage />} />
        <Route path="/tenant/prestador" element={<PrestadorPage />} />
        <Route path="/app" element={<AppLayout />}>
          <Route index element={<Navigate to="documentos" replace />} />
          <Route path=":resourceKey" element={<ResourceLayout />}>
            <Route index element={<ResourceListPage />} />
            <Route path="new" element={<ResourceNewPage />} />
            <Route path=":id" element={<ResourceShowPage />} />
            <Route path=":id/edit" element={<ResourceEditPage />} />
          </Route>
        </Route>
      </Routes>
    </BrowserRouter>
  )
}
