# QuickNFe — comandos locais, qualidade e deploy (Kamal)
#
# Pré-requisitos: Bun, Rust/cargo, cargo-watch, doido CLI, Docker (MySQL local), Kamal (deploy)

.DEFAULT_GOAL := help

BACKEND_DIR   := backend
FRONTEND_DIR  := frontend
KAMAL_CONFIG  := $(BACKEND_DIR)/config/deploy.yml
DOCKER_IMAGE  ?= quicknfe
BUN           ?= bun
COMPOSE       := docker compose

.PHONY: help setup install up down db-create db-migrate db-seed db-setup db-console \
	dev dev-backend dev-frontend dev-sefin-mock build docs \
	lint lint-frontend lint-backend fix fix-frontend fix-backend lint-fix \
	test test-frontend test-backend test-integration test-all ci \
	docker-build deploy upload push

help: ## Lista alvos disponíveis
	@printf "\nQuickNFe — uso: make <alvo>\n\n"
	@grep -E '^[a-zA-Z0-9_.-]+:.*##' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*## "}; {printf "  %-22s %s\n", $$1, $$2}'
	@printf "\nExemplos:\n  make setup && make dev\n  make lint && make test-all\n  make deploy\n\n"

# --- Ambiente local ---

setup: up install db-setup db-seed ## MySQL (Docker), deps e banco dev
install: frontend-install ## Instala dependências do frontend (Bun)
frontend-install:
	cd $(FRONTEND_DIR) && $(BUN) install

up: ## Sobe MySQL (+ Mailpit) em background
	$(COMPOSE) up mysql mailpit -d

down: ## Para containers do compose
	$(COMPOSE) down

down-volumes: ## Para containers do compose
	$(COMPOSE) down --volumes

db-create: ## Cria banco development
	cd $(BACKEND_DIR) && cargo doido db create

db-migrate: ## Migrations development
	cd $(BACKEND_DIR) && cargo doido db migrate

db-seed: ## Seeds development
	cd $(BACKEND_DIR) && cargo doido db seed

db-setup: db-create db-migrate ## create + migrate (dev)

db-console: ## Console MySQL interativo (development; requer `make up`)
	$(COMPOSE) exec mysql mysql -uroot -ppassword quicknfe_development

dev: up db-setup ## Backend :3000 + frontend :3001 (hot reload em ambos)
	$(MAKE) -j2 dev-backend dev-frontend

dev-backend: ## API Doido em http://localhost:3000 (cargo-watch)
	cd $(BACKEND_DIR) && cargo watch -q -w src -w app -w config -w db -x 'doido server'

dev-frontend: ## Vite em http://localhost:3001 (proxy /api → backend)
	cd $(FRONTEND_DIR) && $(BUN) run dev

dev-sefin-mock: ## Mock SEFIN NFS-e em http://127.0.0.1:8090
	cd sefin-mock && cargo run

docs: ## Abre documentação (Swagger requer backend :3000)
	@printf "\nDocs index: docs/README.md\nSwagger UI: http://localhost:3000/api-docs\nOpenAPI: docs/api/openapi.yaml\n\n"

build: ## Build produção (Rust release + frontend estático)
	cd $(BACKEND_DIR) && cargo build --release
	cd $(FRONTEND_DIR) && $(BUN) run build

# --- Lint & fix ---

lint: lint-frontend lint-backend ## Lint frontend (oxlint) + backend (fmt + clippy)

lint-frontend: ## Oxlint no src/
	cd $(FRONTEND_DIR) && $(BUN) run lint

lint-backend: ## rustfmt --check + clippy (tests inclusos)
	cd $(BACKEND_DIR) && cargo fmt -- --check
	cd $(BACKEND_DIR) && cargo clippy --all-targets --tests

fix: fix-frontend fix-backend ## Corrige o que for auto-fixável (lint)

lint-fix: fix ## Alias de fix

fix-frontend: ## oxlint --fix
	cd $(FRONTEND_DIR) && $(BUN) run lint:fix

fix-backend: ## rustfmt + clippy --fix
	cd $(BACKEND_DIR) && cargo fmt
	cd $(BACKEND_DIR) && cargo clippy --fix --allow-dirty --allow-staged --all-targets --tests

# --- Testes ---

test: test-frontend test-backend ## Testes unitários (sem integração MySQL)

test-frontend:
	cd $(FRONTEND_DIR) && $(BUN) run test

test-backend:
	cd $(BACKEND_DIR) && cargo test

test-integration: ## tenant_api_test (MySQL quicknfe_test)
	cd $(BACKEND_DIR) && DOIDO_ENV=test cargo doido db create
	cd $(BACKEND_DIR) && DOIDO_ENV=test cargo doido db migrate
	cd $(BACKEND_DIR) && DOIDO_ENV=test cargo test --test tenant_api_test -- --test-threads=1

test-all: test test-integration ## Unitários + integração API

test-coverage: ## Vitest com threshold 80%
	cd $(FRONTEND_DIR) && $(BUN) run test:coverage

ci: lint test-all build ## Pipeline local (lint + testes + build)

# --- Deploy / upload ---

docker-build: ## Imagem Docker de produção (backend/Dockerfile)
	docker build -t $(DOCKER_IMAGE) $(BACKEND_DIR)

push: docker-build ## Tag local; defina DOCKER_IMAGE=registry/user/quicknfe:tag
	docker push $(DOCKER_IMAGE)

deploy: ## Kamal: build, push registry e sobe serviço (edite config/deploy.yml)
	cd $(BACKEND_DIR) && kamal deploy -c config/deploy.yml

upload: deploy ## Alias: publicar serviço
