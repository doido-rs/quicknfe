-- Seed: emissão simulada de NF-e (modelo 55) — homologação, status AUTORIZADO
-- Pré-requisito: DDL das tabelas empresas … logs_mensagens_retorno já aplicado (PostgreSQL).
--
-- Executar:
--   psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f db/sql/seed_emissao_nfe.sql
--
-- Idempotência: remove apenas os registros deste cenário (CNPJ emissor fixo).

BEGIN;

DELETE FROM logs_mensagens_retorno
WHERE documento_fiscal_id IN (
    SELECT id FROM documentos_fiscais
    WHERE empresa_id = (SELECT id FROM empresas WHERE cpf_cnpj = '11222333000199')
);

DELETE FROM eventos_documento_fiscal
WHERE documento_fiscal_id IN (
    SELECT id FROM documentos_fiscais
    WHERE empresa_id = (SELECT id FROM empresas WHERE cpf_cnpj = '11222333000199')
);

DELETE FROM tributos_item
WHERE item_id IN (
    SELECT i.id
    FROM itens_documento_fiscal i
    JOIN documentos_fiscais d ON d.id = i.documento_fiscal_id
    JOIN empresas e ON e.id = d.empresa_id
    WHERE e.cpf_cnpj = '11222333000199'
);

DELETE FROM itens_documento_fiscal
WHERE documento_fiscal_id IN (
    SELECT d.id
    FROM documentos_fiscais d
    JOIN empresas e ON e.id = d.empresa_id
    WHERE e.cpf_cnpj = '11222333000199'
);

DELETE FROM documentos_fiscais
WHERE empresa_id = (SELECT id FROM empresas WHERE cpf_cnpj = '11222333000199');

DELETE FROM lotes_transmissao
WHERE empresa_id = (SELECT id FROM empresas WHERE cpf_cnpj = '11222333000199');

DELETE FROM clientes
WHERE empresa_id = (SELECT id FROM empresas WHERE cpf_cnpj = '11222333000199');

DELETE FROM certificados_digitais
WHERE empresa_id = (SELECT id FROM empresas WHERE cpf_cnpj = '11222333000199');

DELETE FROM empresas WHERE cpf_cnpj = '11222333000199';

-- 1. Emissor
INSERT INTO empresas (
    cpf_cnpj,
    razao_social,
    inscricao_municipal,
    inscricao_estadual,
    regime_tributario,
    codigo_municipio_ibge,
    uf
) VALUES (
    '11222333000199',
    'QuickNFe Tecnologia LTDA',
    '12345678',
    '123456789012',
    1, -- CRT / regime simplificado para teste
    '3550308',
    'SP'
);

-- 2. Certificado A1 (senha apenas ilustrativa — nunca usar valor real em seed)
INSERT INTO certificados_digitais (
    empresa_id,
    tipo,
    caminho_arquivo,
    senha_encriptada,
    data_validade
)
SELECT
    e.id,
    'A1',
    '/var/secrets/quicknfe/emissor-homolog.pfx',
    'enc:v1:seed-nao-e-segredo',
    TIMESTAMPTZ '2027-12-31 23:59:59+00'
FROM empresas e
WHERE e.cpf_cnpj = '11222333000199';

-- 3. Tomador
INSERT INTO clientes (
    empresa_id,
    cpf_cnpj,
    razao_social,
    inscricao_estadual,
    logradouro,
    numero,
    bairro,
    codigo_municipio_ibge,
    uf,
    email,
    telefone
)
SELECT
    e.id,
    '99888777000166',
    'Cliente Exemplo Comercio LTDA',
    '987654321098',
    'Av. Paulista',
    '1000',
    'Bela Vista',
    '3550308',
    'SP',
    'fiscal@cliente-exemplo.com.br',
    '11999998888'
FROM empresas e
WHERE e.cpf_cnpj = '11222333000199';

-- 4. Lote recebido e processado (SEFAZ homologação)
INSERT INTO lotes_transmissao (
    empresa_id,
    numero_lote,
    protocolo_recebimento,
    ambiente,
    status_lote,
    data_recebimento
)
SELECT
    e.id,
    1,
    '135260924123456',
    2,
    'Processado',
    TIMESTAMPTZ '2026-09-24 14:05:10-03'
FROM empresas e
WHERE e.cpf_cnpj = '11222333000199';

-- 5. NF-e autorizada (série 1, número 1)
-- Chave fictícia de 44 dígitos (homologação); substituir por chave válida em testes de integração.
INSERT INTO documentos_fiscais (
    empresa_id,
    cliente_id,
    lote_id,
    modelo,
    serie,
    numero_documento,
    chave_acesso,
    ambiente,
    tipo_emissao,
    status,
    data_emissao,
    codigo_municipio_emissao,
    protocolo_autorizacao,
    valor_total_servicos_produtos,
    valor_liquido,
    xml_assinado,
    xml_processado
)
SELECT
    e.id,
    c.id,
    l.id,
    '55',
    '1',
    1,
    '35260911222333000199550010000000001123456789',
    2,
    1,
    'AUTORIZADO',
    TIMESTAMPTZ '2026-09-24 14:04:55-03',
    '3550308',
    '135260924123457',
    1500.00,
    1500.00,
    '<?xml version="1.0" encoding="UTF-8"?><NFe xmlns="http://www.portalfiscal.inf.br/nfe"><!-- assinado (seed) --></NFe>',
    '<?xml version="1.0" encoding="UTF-8"?><nfeProc versao="4.00"><!-- procNFe (seed) --></nfeProc>'
FROM empresas e
JOIN clientes c ON c.empresa_id = e.id AND c.cpf_cnpj = '99888777000166'
JOIN lotes_transmissao l ON l.empresa_id = e.id AND l.numero_lote = 1
WHERE e.cpf_cnpj = '11222333000199';

-- 6. Itens
INSERT INTO itens_documento_fiscal (
    documento_fiscal_id,
    numero_item,
    descricao,
    codigo_ncm,
    quantidade,
    valor_unitario,
    valor_total_bruto,
    valor_desconto
)
SELECT
    d.id,
    1,
    'Licença de software QuickNFe — plano anual',
    '85234990',
    1.0000,
    1000.0000,
    1000.00,
    0.00
FROM documentos_fiscais d
JOIN empresas e ON e.id = d.empresa_id
WHERE e.cpf_cnpj = '11222333000199'
  AND d.modelo = '55'
  AND d.serie = '1'
  AND d.numero_documento = 1;

INSERT INTO itens_documento_fiscal (
    documento_fiscal_id,
    numero_item,
    descricao,
    codigo_ncm,
    quantidade,
    valor_unitario,
    valor_total_bruto,
    valor_desconto
)
SELECT
    d.id,
    2,
    'Suporte técnico especializado (12 meses)',
    '00000000',
    1.0000,
    500.0000,
    500.00,
    0.00
FROM documentos_fiscais d
JOIN empresas e ON e.id = d.empresa_id
WHERE e.cpf_cnpj = '11222333000199'
  AND d.modelo = '55'
  AND d.serie = '1'
  AND d.numero_documento = 1;

-- 7. Tributos por item
-- Nota: o DDL atual não discrimina o imposto (ICMS/PIS/…); use uma linha por tributo no item.
-- Item 1 — ICMS CSOSN 102 (Simples, sem crédito)
INSERT INTO tributos_item (
    item_id,
    cst_csosn,
    base_calculo,
    aliquota,
    valor_imposto,
    iss_retido,
    classificacao_trib_ibscbs,
    aliquota_ibs,
    valor_ibs,
    aliquota_cbs,
    valor_cbs
)
SELECT
    i.id,
    '102',
    1000.00,
    0.0000,
    0.00,
    FALSE,
    NULL,
    0.1200,
    120.00,
    0.0900,
    90.00
FROM itens_documento_fiscal i
JOIN documentos_fiscais d ON d.id = i.documento_fiscal_id
JOIN empresas e ON e.id = d.empresa_id
WHERE e.cpf_cnpj = '11222333000199'
  AND i.numero_item = 1;

INSERT INTO tributos_item (
    item_id,
    cst_csosn,
    base_calculo,
    aliquota,
    valor_imposto,
    iss_retido
)
SELECT
    i.id,
    '49',
    1000.00,
    0.6500,
    6.50,
    FALSE
FROM itens_documento_fiscal i
JOIN documentos_fiscais d ON d.id = i.documento_fiscal_id
JOIN empresas e ON e.id = d.empresa_id
WHERE e.cpf_cnpj = '11222333000199'
  AND i.numero_item = 1;

INSERT INTO tributos_item (
    item_id,
    cst_csosn,
    base_calculo,
    aliquota,
    valor_imposto,
    iss_retido
)
SELECT
    i.id,
    '49',
    1000.00,
    3.0000,
    30.00,
    FALSE
FROM itens_documento_fiscal i
JOIN documentos_fiscais d ON d.id = i.documento_fiscal_id
JOIN empresas e ON e.id = d.empresa_id
WHERE e.cpf_cnpj = '11222333000199'
  AND i.numero_item = 1;

-- Item 2 — serviço: ISSQN retido na fonte (simulado)
INSERT INTO tributos_item (
    item_id,
    cst_csosn,
    base_calculo,
    aliquota,
    valor_imposto,
    iss_retido
)
SELECT
    i.id,
    NULL,
    500.00,
    5.0000,
    25.00,
    TRUE
FROM itens_documento_fiscal i
JOIN documentos_fiscais d ON d.id = i.documento_fiscal_id
JOIN empresas e ON e.id = d.empresa_id
WHERE e.cpf_cnpj = '11222333000199'
  AND i.numero_item = 2;

-- 8. Log de retorno SEFAZ (cStat 100)
INSERT INTO logs_mensagens_retorno (
    documento_fiscal_id,
    lote_id,
    codigo_erro,
    mensagem_retorno,
    correcao_sugerida
)
SELECT
    d.id,
    d.lote_id,
    '100',
    'Autorizado o uso da NF-e',
    NULL
FROM documentos_fiscais d
JOIN empresas e ON e.id = d.empresa_id
WHERE e.cpf_cnpj = '11222333000199'
  AND d.modelo = '55'
  AND d.numero_documento = 1;

COMMIT;

-- Conferência rápida
SELECT
    e.razao_social AS emissor,
    d.modelo,
    d.serie,
    d.numero_documento,
    d.status,
    d.chave_acesso,
    d.protocolo_autorizacao,
    COUNT(i.id) AS qtd_itens
FROM documentos_fiscais d
JOIN empresas e ON e.id = d.empresa_id
LEFT JOIN itens_documento_fiscal i ON i.documento_fiscal_id = d.id
WHERE e.cpf_cnpj = '11222333000199'
GROUP BY e.razao_social, d.modelo, d.serie, d.numero_documento, d.status, d.chave_acesso, d.protocolo_autorizacao;
