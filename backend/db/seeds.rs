//! Database seeds — run with `cargo doido db migrate && cargo doido db seed`.

use doido::model::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, FromQueryResult,
    QueryFilter, QueryResult, Set, Statement,
};
use doido::model::sea_orm::prelude::{DateTimeUtc, Decimal};

use crate::models::certificado_digital::ActiveModel as CertificadoActive;
use crate::models::cna::{Column as CnaColumn, Entity as CnaEntity};
use crate::models::cliente::{ActiveModel as ClienteActive, Column as ClienteColumn, Entity as ClienteEntity};
use crate::models::company::{
    ActiveModel as CompanyActive, Column as CompanyColumn, Entity as CompanyEntity,
};
use crate::models::municipio::ActiveModel as MunicipioActive;
use crate::models::company_usuario::ActiveModel as CompanyMembroActive;
use crate::models::documento_fiscal::{
    ActiveModel as DocumentoActive, Column as DocumentoColumn, Entity as DocumentoEntity,
};
use crate::models::empresa::{
    ActiveModel as EmpresaActive, Column as EmpresaColumn, Entity as EmpresaEntity,
};
use crate::models::user::{ActiveModel as UserActive, Entity as UserEntity};
use doido::model::password::hash_password;
use crate::models::item_documento_fiscal::{
    ActiveModel as ItemActive, Column as ItemColumn, Entity as ItemEntity,
};
use crate::models::log_mensagem_retorno::ActiveModel as LogActive;
use crate::models::lote_transmissao::ActiveModel as LoteActive;
use crate::models::municipio::{Column as MunicipioColumn, Entity as MunicipioEntity};
use crate::models::tributo_item::ActiveModel as TributoActive;

const CNPJ_EMISSOR: &str = "11222333000199";
/// Prestador seedado a partir de consulta CNC (Rio de Janeiro / IBGE 3304557).
const CNPJ_PRESTADOR_BAYAN_CNC: &str = "31041140000131";
const DEMO_COMPANY_NAME: &str = "QuickNFe Demo";
const SEED_USER_EMAIL: &str = "admin@quicknfe.local";
/// Documented in README / docs/guides/getting-started.md
const SEED_USER_PASSWORD: &str = "QuickNFe2026!";
const CNA_CODIGO_DEMO_ITEM: &str = "010101";

struct SchemaColCount {
    n: i64,
}

impl FromQueryResult for SchemaColCount {
    fn from_query_result(row: &QueryResult, pre: &str) -> Result<Self, DbErr> {
        Ok(Self {
            n: row.try_get(pre, "n")?,
        })
    }
}

/// Endereço Brasil + `processamento_mensagem` exigem migrations recentes antes do seed.
async fn assert_seed_schema(db: &DatabaseConnection) -> doido::Result<()> {
    let row = SchemaColCount::find_by_statement(Statement::from_string(
        db.get_database_backend(),
        "SELECT COUNT(*) AS n FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'empresas' AND COLUMN_NAME = 'cep'"
            .to_string(),
    ))
    .one(db)
    .await?;

    if row.map(|r| r.n).unwrap_or(0) == 0 {
        return Err(DbErr::Custom(
            "schema desatualizado: coluna empresas.cep ausente. \
             Rode `cargo doido db migrate` (ou `make db-migrate`) antes de `cargo doido db seed`."
                .into(),
        )
        .into());
    }

    Ok(())
}

fn dec(value: i64) -> Decimal {
    Decimal::from(value)
}

fn dec_frac(int: i64, scale: u32) -> Decimal {
    Decimal::new(int, scale)
}

async fn table_exists(db: &DatabaseConnection, table_name: &str) -> doido::Result<bool> {
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT COUNT(*) AS n FROM information_schema.TABLES \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = '{table_name}'"
    );
    let row = SchemaColCount::find_by_statement(Statement::from_string(backend, sql))
        .one(db)
        .await?;
    Ok(row.map(|r| r.n).unwrap_or(0) > 0)
}

/// Parametrização municipal (ADN) ainda não carregada — sinal para bootstrap do catálogo.
async fn municipio_cna_parametrizacao_is_empty(db: &DatabaseConnection) -> doido::Result<bool> {
    if !table_exists(db, "municipio_cna_parametrizacao").await? {
        return Ok(true);
    }
    let backend = db.get_database_backend();
    let row = SchemaColCount::find_by_statement(Statement::from_string(
        backend,
        "SELECT COUNT(*) AS n FROM municipio_cna_parametrizacao".to_string(),
    ))
    .one(db)
    .await?;
    Ok(row.map(|r| r.n).unwrap_or(0) == 0)
}

async fn pending_doido_job_count(
    db: &DatabaseConnection,
    queue: &str,
    job_name: &str,
) -> doido::Result<i64> {
    let backend = db.get_database_backend();
    let pattern = format!("%{job_name}%");
    let sql = format!(
        "SELECT COUNT(*) AS n FROM doido_jobs \
         WHERE queue = '{queue}' AND status IN ('pending','scheduled') \
         AND data LIKE '{pattern}'"
    );
    let pending = SchemaColCount::find_by_statement(Statement::from_string(backend, sql))
        .one(db)
        .await?;
    Ok(pending.map(|r| r.n).unwrap_or(0))
}

/// Enfileira Anexo A (municípios/países) e sync diário do catálogo (`cargo doido worker`).
async fn bootstrap_nfse_catalog_jobs(db: &DatabaseConnection) -> doido::Result<()> {
    if !municipio_cna_parametrizacao_is_empty(db).await? {
        return Ok(());
    }

    let queue = crate::services::job_queue::queue_for(db.clone()).await?;

    if pending_doido_job_count(db, "default", "import_nfse_anexo_a").await? == 0 {
        use crate::jobs::import_nfse_anexo_a::{
            ImportNfseAnexoAJob, ImportNfseAnexoAJobPayload,
        };
        ImportNfseAnexoAJob::new(ImportNfseAnexoAJobPayload::default())
            .enqueue(queue.as_ref())
            .await
            .map_err(|e| DbErr::Custom(format!("enqueue import anexo A: {e}")))?;
    }

    if pending_doido_job_count(db, "catalog", "sync_nfse_catalog_daily").await? == 0 {
        use crate::jobs::sync_nfse_catalog_daily::{
            SyncNfseCatalogDailyJob, SyncNfseCatalogDailyJobPayload,
        };
        SyncNfseCatalogDailyJob::new(SyncNfseCatalogDailyJobPayload::default())
            .enqueue(queue.as_ref())
            .await
            .map_err(|e| DbErr::Custom(format!("enqueue catalog sync: {e}")))?;
    }

    Ok(())
}

pub async fn run(db: &DatabaseConnection) -> doido::Result<()> {
    assert_seed_schema(db).await?;
    bootstrap_nfse_catalog_jobs(db).await?;
    seed_prestador_bayan_cnc(db).await?;
    repair_seed_fixture_data(db).await?;

    if UserEntity::find()
        .filter(crate::models::user::Column::Email.eq(SEED_USER_EMAIL))
        .one(db)
        .await?
        .is_some()
    {
        return Ok(());
    }

    let empresa = if let Some(existing) = EmpresaEntity::find()
        .filter(crate::models::empresa::Column::CpfCnpj.eq(CNPJ_EMISSOR))
        .one(db)
        .await?
    {
        existing
    } else {
        seed_fiscal_demo(db).await?
    };

    let company_id = empresa.company_id;

    let now: DateTimeUtc = chrono::Utc::now();
    let user = UserActive {
        email: Set(SEED_USER_EMAIL.into()),
        password_digest: Set(hash_password(SEED_USER_PASSWORD)?),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;

    CompanyMembroActive {
        user_id: Set(user.id),
        company_id: Set(company_id),
        papel: Set("proprietario".into()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}

async fn ensure_demo_company(
    db: &DatabaseConnection,
) -> doido::Result<crate::models::company::Model> {
    if let Some(existing) = CompanyEntity::find()
        .filter(CompanyColumn::Nome.eq(DEMO_COMPANY_NAME))
        .one(db)
        .await?
    {
        return Ok(existing);
    }

    CompanyActive {
        nome: Set(DEMO_COMPANY_NAME.into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(Into::into)
}

async fn ensure_municipio(
    db: &DatabaseConnection,
    codigo_ibge: &str,
    nome: &str,
    uf: &str,
) -> doido::Result<crate::models::municipio::Model> {
    if let Some(existing) = MunicipioEntity::find()
        .filter(MunicipioColumn::CodigoIbge.eq(codigo_ibge))
        .one(db)
        .await?
    {
        return Ok(existing);
    }

    MunicipioActive {
        codigo_ibge: Set(codigo_ibge.into()),
        nome: Set(nome.into()),
        uf: Set(uf.into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(Into::into)
}

/// Dados espelhados de `InfCad` na consulta CNC (OrigemDados 3, CodigoMunicipio 3304557).
/// OpcaoSimplesNacional 0 → regime `REGIME_NORMAL` (não optante informado).
async fn seed_prestador_bayan_cnc(db: &DatabaseConnection) -> doido::Result<()> {
    if EmpresaEntity::find()
        .filter(crate::models::empresa::Column::CpfCnpj.eq(CNPJ_PRESTADOR_BAYAN_CNC))
        .one(db)
        .await?
        .is_some()
    {
        return Ok(());
    }

    let company = ensure_demo_company(db).await?;
    let municipio_rj = ensure_municipio(db, "3304557", "Rio de Janeiro", "RJ").await?;
    let now: DateTimeUtc = chrono::Utc::now();

    EmpresaActive {
        company_id: Set(company.id),
        cpf_cnpj: Set(CNPJ_PRESTADOR_BAYAN_CNC.into()),
        razao_social: Set("BAYAN CODE DESENVOLVIMENTO DE SOFTWARE LTDA".into()),
        inscricao_municipal: Set(None),
        inscricao_estadual: Set(None),
        regime_tributario: Set("REGIME_NORMAL".into()),
        municipio_id: Set(municipio_rj.id),
        cep: Set(Some("23075350".into())),
        logradouro: Set(Some("Rua Ivan Serpa".into())),
        numero: Set(Some("451".into())),
        bairro: Set(Some("Campo Grande".into())),
        criado_em: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}

async fn cna_demo_consultoria(db: &DatabaseConnection) -> doido::Result<i64> {
    Ok(CnaEntity::find()
        .filter(CnaColumn::Codigo.eq(CNA_CODIGO_DEMO_ITEM))
        .one(db)
        .await?
        .expect("CNA 010101 — rode as migrations")
        .id)
}

/// Atualiza registros seedados antes das colunas de endereço / CNA (idempotente).
async fn repair_seed_fixture_data(db: &DatabaseConnection) -> doido::Result<()> {
    if let Some(empresa) = EmpresaEntity::find()
        .filter(EmpresaColumn::CpfCnpj.eq(CNPJ_PRESTADOR_BAYAN_CNC))
        .one(db)
        .await?
    {
        if empresa.cep.as_deref().unwrap_or("").is_empty() {
            let mut active: EmpresaActive = empresa.into();
            active.cep = Set(Some("23075350".into()));
            active.logradouro = Set(Some("Rua Ivan Serpa".into()));
            active.numero = Set(Some("451".into()));
            active.bairro = Set(Some("Campo Grande".into()));
            active.update(db).await?;
        }
    }

    if let Some(empresa) = EmpresaEntity::find()
        .filter(EmpresaColumn::CpfCnpj.eq(CNPJ_EMISSOR))
        .one(db)
        .await?
    {
        let empresa_id = empresa.id;

        if empresa.cep.as_deref().unwrap_or("").is_empty() {
            let mut active: EmpresaActive = empresa.into();
            active.cep = Set(Some("01310100".into()));
            active.logradouro = Set(Some("Avenida Paulista".into()));
            active.numero = Set(Some("1000".into()));
            active.bairro = Set(Some("Bela Vista".into()));
            active.update(db).await?;
        }

        if let Some(cliente) = ClienteEntity::find()
            .filter(ClienteColumn::EmpresaId.eq(empresa_id))
            .filter(ClienteColumn::CpfCnpj.eq("12345678901"))
            .one(db)
            .await?
        {
            let needs_addr = cliente.cep.as_deref().unwrap_or("").is_empty()
                || cliente.logradouro.as_deref().unwrap_or("").is_empty();
            if needs_addr {
                let mut active: ClienteActive = cliente.into();
                active.cep = Set(Some("01310100".into()));
                active.logradouro = Set(Some("Rua Augusta".into()));
                active.numero = Set(Some("500".into()));
                active.bairro = Set(Some("Consolação".into()));
                active.update(db).await?;
            }
        }

        let doc_ids: Vec<i64> = DocumentoEntity::find()
            .filter(DocumentoColumn::EmpresaId.eq(empresa_id))
            .all(db)
            .await?
            .into_iter()
            .map(|d| d.id)
            .collect();

        if !doc_ids.is_empty() {
            let cna_id = cna_demo_consultoria(db).await?;
            let itens = ItemEntity::find()
                .filter(ItemColumn::DocumentoFiscalId.is_in(doc_ids))
                .all(db)
                .await?;

            for item in itens {
                if item.codigo_tributacao_nacional.is_none() && item.cna_id.is_none() {
                    let mut active: ItemActive = item.into();
                    active.codigo_tributacao_nacional = Set(Some(CNA_CODIGO_DEMO_ITEM.into()));
                    active.cna_id = Set(Some(cna_id));
                    active.update(db).await?;
                }
            }
        }
    }

    Ok(())
}

async fn seed_fiscal_demo(db: &DatabaseConnection) -> doido::Result<crate::models::empresa::Model> {
    let data_emissao: DateTimeUtc = "2026-09-24T17:04:55Z".parse().unwrap();
    let data_recebimento: DateTimeUtc = "2026-09-24T17:05:10Z".parse().unwrap();
    let validade_cert: DateTimeUtc = "2027-12-31T23:59:59Z".parse().unwrap();

    let company = ensure_demo_company(db).await?;

    let municipio_sp = MunicipioEntity::find()
        .filter(MunicipioColumn::CodigoIbge.eq("3550308"))
        .one(db)
        .await?
        .expect("município São Paulo (3550308) — rode as migrations");

    let empresa = EmpresaActive {
        company_id: Set(company.id),
        cpf_cnpj: Set(CNPJ_EMISSOR.into()),
        razao_social: Set("QuickNFe Tecnologia LTDA".into()),
        inscricao_municipal: Set(Some("12345678".into())),
        inscricao_estadual: Set(Some("123456789012".into())),
        regime_tributario: Set("SIMPLES_NACIONAL".into()),
        municipio_id: Set(municipio_sp.id),
        cep: Set(Some("01310100".into())),
        logradouro: Set(Some("Avenida Paulista".into())),
        numero: Set(Some("1000".into())),
        bairro: Set(Some("Bela Vista".into())),
        criado_em: Set(chrono::Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    CertificadoActive {
        empresa_id: Set(empresa.id),
        tipo: Set("A1".into()),
        caminho_arquivo: Set(Some(
            "/var/secrets/quicknfe/emissor-homolog.pfx".into(),
        )),
        senha_encriptada: Set("enc:v1:seed-nao-e-segredo".into()),
        data_validade: Set(validade_cert),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let cliente = ClienteActive {
        empresa_id: Set(empresa.id),
        cpf_cnpj: Set("12345678901".into()),
        razao_social: Set("Cliente Exemplo".into()),
        inscricao_municipal: Set(None),
        inscricao_estadual: Set(None),
        cep: Set(Some("01310100".into())),
        logradouro: Set(Some("Rua Augusta".into())),
        numero: Set(Some("500".into())),
        bairro: Set(Some("Consolação".into())),
        municipio_id: Set(Some(municipio_sp.id)),
        email: Set(Some("fiscal@cliente-exemplo.com.br".into())),
        telefone: Set(None),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let lote = LoteActive {
        empresa_id: Set(empresa.id),
        numero_lote: Set(1),
        protocolo_recebimento: Set(Some("PROT-SEED-001".into())),
        ambiente: Set(2),
        status_lote: Set("processado".into()),
        data_recebimento: Set(Some(data_recebimento)),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let documento = DocumentoActive {
        empresa_id: Set(empresa.id),
        cliente_id: Set(Some(cliente.id)),
        lote_id: Set(Some(lote.id)),
        modelo: Set("NFSe".into()),
        serie: Set("1".into()),
        numero_documento: Set(1001),
        chave_acesso: Set(None),
        codigo_verificacao: Set(None),
        ambiente: Set(2),
        tipo_emissao: Set(1),
        status: Set("autorizada".into()),
        data_emissao: Set(data_emissao),
        data_competencia: Set(Some(
            chrono::NaiveDate::from_ymd_opt(2026, 9, 1).expect("data competência seed"),
        )),
        municipio_emissao_id: Set(municipio_sp.id),
        municipio_prestacao_id: Set(municipio_sp.id),
        protocolo_autorizacao: Set(Some("AUTH-SEED".into())),
        valor_total_servicos_produtos: Set(dec(1500)),
        valor_liquido: Set(dec(1500)),
        xml_assinado: Set(None),
        xml_processado: Set(None),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let cna_id = cna_demo_consultoria(db).await?;

    let item = ItemActive {
        documento_fiscal_id: Set(documento.id),
        numero_item: Set(1),
        descricao: Set("Análise e desenvolvimento de sistemas".into()),
        descricao_servico: Set(Some("Análise e desenvolvimento de sistemas".into())),
        codigo_ncm: Set(None),
        codigo_tributacao_nacional: Set(Some(CNA_CODIGO_DEMO_ITEM.into())),
        cna_id: Set(Some(cna_id)),
        quantidade: Set(dec_frac(1, 0)),
        valor_unitario: Set(dec(1500)),
        valor_total_bruto: Set(dec(1500)),
        valor_desconto: Set(None),
        ..Default::default()
    }
    .insert(db)
    .await?;

    TributoActive {
        item_id: Set(item.id),
        tipo_imposto: Set("ISS".into()),
        cst_csosn: Set(None),
        base_calculo: Set(Some(dec(1500))),
        aliquota: Set(Some(dec_frac(5, 2))),
        valor_imposto: Set(Some(dec(75))),
        iss_retido: Set(Some(0)),
        classificacao_trib_ibscbs: Set(None),
        aliquota_ibs: Set(Some(dec(0))),
        valor_ibs: Set(Some(dec(0))),
        aliquota_cbs: Set(Some(dec(0))),
        valor_cbs: Set(Some(dec(0))),
        ..Default::default()
    }
    .insert(db)
    .await?;

    LogActive {
        documento_fiscal_id: Set(Some(documento.id)),
        lote_id: Set(Some(lote.id)),
        codigo_erro: Set("100".into()),
        mensagem_retorno: Set("NFSe autorizada (ambiente de homologação)".into()),
        correcao_sugerida: Set(None),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(empresa)
}
