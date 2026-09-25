# QuickNFe — Frontend

SPA React + Vite para gestão multi-empresa via API do backend Doido.

**UI:** [Tailwind CSS v4](https://tailwindcss.com) + [shadcn/ui](https://ui.shadcn.com) (componentes em `src/components/ui/`). Layout autenticado com **sidebar** (`AppSidebar` + `SidebarProvider`).

## Requisitos

- [Bun](https://bun.sh) 1.1+
- Backend em `http://localhost:3000` (ver [backend/README.md](../backend/README.md))

Atalhos na raiz do repo: `make setup`, `make dev`, `make lint`, `make fix`, `make test` — ver [Makefile](../Makefile).

## Desenvolvimento

```bash
cd frontend
bun install
bun run dev
```

Abra [http://localhost:5173](http://localhost:5173). O Vite faz proxy de `/api`, `/users` e `/tenant` para o backend.

Variável opcional: `VITE_API_URL` — deixe vazio para usar o proxy.

### Adicionar componentes shadcn

```bash
bunx shadcn@latest add dialog
```

Configuração em [`components.json`](components.json). Imports usam alias `@/` → `src/`.

## Credenciais demo (seed)

- E-mail: `admin@quicknfe.local`
- Senha: `QuickNFe2026!`

Fluxo: login → escolher empresa → editar recursos no menu lateral.

## Testes e coverage

```bash
bun run test
bun run test:coverage
```

Meta de coverage global: **80%** (linhas, funções, branches, statements). Componentes shadcn em `src/components/ui/` ficam fora do escopo de cobertura.

## Build

```bash
bun run build
bun run preview
```
