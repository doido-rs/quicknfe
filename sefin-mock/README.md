# sefin-mock

Servidor [Axum](https://github.com/tokio-rs/axum) que simula a API **SEFIN Nacional** (NFS-e) para desenvolvimento local, sem depender da produção restrita da SEFAZ.

## Endpoints

| Método | Caminho | Descrição |
|--------|---------|-----------|
| `POST` | `/SefinNacional/nfse` | Emissão (body JSON `{ "dpsXmlGZipB64": "..." }`) → **201** |
| `GET` | `/SefinNacional/dps/{id}` | Consulta DPS mockada processada |
| `GET` | `/health` | Health check |

Os mesmos paths existem com prefixo **`/API/SefinNacional`** (compatível com o cliente `quicknfe` em homologação).

Porta padrão: **8090** (`SEFIN_MOCK_PORT`).

## Executar

```bash
cd sefin-mock
cargo run
```

## Integrar com quicknfe

Com o mock rodando:

No `.env` (development) ou no YAML:

```bash
# .env — lido via settings.sefin.base_url_homologacao em config/development.yml
SEFIN_BASE_URL=http://127.0.0.1:8090/API/SefinNacional
cd backend && cargo doido server
```

Documentos com `ambiente: 2` (homologação) passam a enviar a DPS para o mock. O certificado mTLS ainda é enviado pelo cliente, mas o mock **não exige** validação de certificado.

## Comportamento

- Descompacta `dpsXmlGZipB64`, exige presença de `<Signature>` na DPS.
- Extrai `Id` de `infDPS` quando existir; senão gera um id `DPS{uuid}`.
- Persiste em memória e responde com `chaveAcesso` mock (44 dígitos) e `nfseXmlGZipB64`.
- `GET /dps/{id}` devolve status `PROCESSADO` e os XMLs compactados.

## Testes

```bash
cargo test
```
