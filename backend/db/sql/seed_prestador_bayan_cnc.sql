-- Prestador de serviço a partir de resposta CNC (OrigemDados 3).
-- InfCad: CNPJ 31041140000131, Rio de Janeiro (IBGE 3304557, UF 33).
-- Referência CNC (não persistidos em `empresas`):
--   Email fabiano.paula.martins@gmail.com
--   Telefone (21)99851-9993
--   CEP 23075-350 — RUA IVAN SERPA, 451, CAMPO GRANDE
--   OpcaoSimplesNacional 0 → regime REGIME_NORMAL
--
-- MySQL (Doido). Idempotente por CPF/CNPJ.
--   cargo doido db migrate && cargo doido db seed
-- ou execute este arquivo após migrations.

INSERT INTO municipios (codigo_ibge, nome, uf)
VALUES ('3304557', 'Rio de Janeiro', 'RJ')
ON DUPLICATE KEY UPDATE
    nome = VALUES(nome),
    uf = VALUES(uf);

INSERT INTO companies (nome, criado_em)
SELECT 'QuickNFe Demo', UTC_TIMESTAMP()
WHERE NOT EXISTS (
    SELECT 1 FROM companies WHERE nome = 'QuickNFe Demo' LIMIT 1
);

INSERT INTO empresas (
    company_id,
    cpf_cnpj,
    razao_social,
    inscricao_municipal,
    inscricao_estadual,
    regime_tributario,
    municipio_id,
    criado_em
)
SELECT
    c.id,
    '31041140000131',
    'BAYAN CODE DESENVOLVIMENTO DE SOFTWARE LTDA',
    NULL,
    NULL,
    'REGIME_NORMAL',
    m.id,
    UTC_TIMESTAMP()
FROM companies c
CROSS JOIN municipios m
WHERE c.nome = 'QuickNFe Demo'
  AND m.codigo_ibge = '3304557'
  AND NOT EXISTS (
      SELECT 1 FROM empresas e WHERE e.cpf_cnpj = '31041140000131' LIMIT 1
  );
