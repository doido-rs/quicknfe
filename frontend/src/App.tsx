import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { AppLayout } from './pages/AppLayout'
import { LoginPage } from './pages/LoginPage'
import { RegisterPage } from './pages/RegisterPage'
import { ResourcePage } from './pages/ResourcePage'
import { TenantPage } from './pages/TenantPage'
export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Navigate to="/login" replace />} />
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/tenant" element={<TenantPage />} />
        <Route path="/app" element={<AppLayout />}>
          <Route index element={<Navigate to="empresa" replace />} />
          <Route path=":resourceKey" element={<ResourcePage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}
