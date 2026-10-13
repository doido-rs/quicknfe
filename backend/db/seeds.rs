//! Database seeds — run with `cargo doido db seed`.

use doido::model::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use doido::model::sea_orm::prelude::{DateTimeUtc, Decimal};

use crate::models::certificado_digital::ActiveModel as CertificadoActive;
use crate::models::cliente::ActiveModel as ClienteActive;
use crate::models::company::{
    ActiveModel as CompanyActive, Column as CompanyColumn, Entity as CompanyEntity,
};
use crate::models::municipio::ActiveModel as MunicipioActive;
use crate::models::company_usuario::ActiveModel as CompanyMembroActive;
use crate::models::documento_fiscal::ActiveModel as DocumentoActive;
use crate::models::empresa::{ActiveModel as EmpresaActive, Entity as EmpresaEntity};
use crate::models::user::{ActiveModel as UserActive, Entity as UserEntity};
use doido::model::password::hash_password;
use crate::models::item_documento_fiscal::ActiveModel as ItemActive;
use crate::models::log_mensagem_retorno::ActiveModel as LogActive;
use crate::models::lote_transmissao::ActiveModel as LoteActive;
use crate::models::municipio::{Column as MunicipioColumn, Entity as MunicipioEntity};
use crate::models::tributo_item::ActiveModel as TributoActive;

const CNPJ_EMISSOR: &str = "11222333000199";
/// Prestador seedado a partir de consulta CNC (Rio de Janeiro / IBGE 3304557).
const CNPJ_PRESTADOR_BAYAN_CNC: &str = "31041140000131";
const DEMO_COMPANY_NAME: &str = "QuickNFe Demo";
const SEED_USER_EMAIL: &str = "admin@quicknfe.local";

fn dec(value: i64) -> Decimal {
    Decimal::from(value)
}

fn dec_frac(int: i64, scale: u32) -> Decimal {
    Decimal::new(int, scale)
}

pub async fn run(db: &DatabaseConnection) -> doido::Result<()> {
    seed_prestador_bayan_cnc(db).await?;

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
        password_digest: Set(hash_password("Admin@1234")?),
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
        criado_em: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;

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
        logradouro: Set(None),
        numero: Set(None),
        bairro: Set(None),
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
        data_competencia: Set(None),
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

    let item = ItemActive {
        documento_fiscal_id: Set(documento.id),
        numero_item: Set(1),
        descricao: Set("Consultoria em NF-e".into()),
        descricao_servico: Set(Some("Consultoria em NF-e".into())),
        codigo_ncm: Set(None),
        codigo_tributacao_nacional: Set(None),
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
        codigo_erro: Set("0".into()),
        mensagem_retorno: Set("Autorizado".into()),
        correcao_sugerida: Set(None),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(empresa)
}
