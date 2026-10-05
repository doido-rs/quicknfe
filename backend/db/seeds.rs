//! Database seeds — run with `cargo doido db seed`.

use doido::model::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use doido::model::sea_orm::prelude::{DateTimeUtc, Decimal};

use crate::models::certificado_digital::ActiveModel as CertificadoActive;
use crate::models::cliente::ActiveModel as ClienteActive;
use crate::models::documento_fiscal::ActiveModel as DocumentoActive;
use crate::models::empresa::{ActiveModel as EmpresaActive, Entity as EmpresaEntity};
use crate::models::empresa_usuario::ActiveModel as MembroActive;
use crate::models::user::{ActiveModel as UserActive, Entity as UserEntity};
use doido::model::password::hash_password;
use crate::models::item_documento_fiscal::ActiveModel as ItemActive;
use crate::models::log_mensagem_retorno::ActiveModel as LogActive;
use crate::models::lote_transmissao::ActiveModel as LoteActive;
use crate::models::tributo_item::ActiveModel as TributoActive;

const CNPJ_EMISSOR: &str = "11222333000199";
const SEED_USER_EMAIL: &str = "admin@quicknfe.local";

fn dec(value: i64) -> Decimal {
    Decimal::from(value)
}

fn dec_frac(int: i64, scale: u32) -> Decimal {
    Decimal::new(int, scale)
}

pub async fn run(db: &DatabaseConnection) -> doido::Result<()> {
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

    MembroActive {
        user_id: Set(user.id),
        empresa_id: Set(empresa.id),
        papel: Set("proprietario".into()),
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

    let empresa = EmpresaActive {
        cpf_cnpj: Set(CNPJ_EMISSOR.into()),
        razao_social: Set("QuickNFe Tecnologia LTDA".into()),
        inscricao_municipal: Set(Some("12345678".into())),
        inscricao_estadual: Set(Some("123456789012".into())),
        regime_tributario: Set(1),
        codigo_municipio_ibge: Set("3550308".into()),
        uf: Set("SP".into()),
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
        cpf_cnpj: Set("99888777000166".into()),
        razao_social: Set("Cliente Exemplo Comercio LTDA".into()),
        inscricao_estadual: Set(Some("987654321098".into())),
        logradouro: Set(Some("Av. Paulista".into())),
        numero: Set(Some("1000".into())),
        bairro: Set(Some("Bela Vista".into())),
        codigo_municipio_ibge: Set(Some("3550308".into())),
        uf: Set(Some("SP".into())),
        email: Set(Some("fiscal@cliente-exemplo.com.br".into())),
        telefone: Set(Some("11999998888".into())),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let lote = LoteActive {
        empresa_id: Set(empresa.id),
        numero_lote: Set(1),
        protocolo_recebimento: Set(Some("135260924123456".into())),
        ambiente: Set(2),
        status_lote: Set("Processado".into()),
        data_recebimento: Set(Some(data_recebimento)),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let documento = DocumentoActive {
        empresa_id: Set(empresa.id),
        cliente_id: Set(Some(cliente.id)),
        lote_id: Set(Some(lote.id)),
        modelo: Set("55".into()),
        serie: Set("1".into()),
        numero_documento: Set(1),
        chave_acesso: Set(Some(
            "35260911222333000199550010000000001123456789".into(),
        )),
        ambiente: Set(2),
        tipo_emissao: Set(1),
        status: Set("AUTORIZADO".into()),
        data_emissao: Set(data_emissao),
        codigo_municipio_emissao: Set("3550308".into()),
        protocolo_autorizacao: Set(Some("135260924123457".into())),
        valor_total_servicos_produtos: Set(dec(1500)),
        valor_liquido: Set(dec(1500)),
        xml_assinado: Set(Some(
            r#"<?xml version="1.0" encoding="UTF-8"?><NFe xmlns="http://www.portalfiscal.inf.br/nfe"><!-- seed --></NFe>"#
                .into(),
        )),
        xml_processado: Set(Some(
            r#"<?xml version="1.0" encoding="UTF-8"?><nfeProc versao="4.00"><!-- seed --></nfeProc>"#
                .into(),
        )),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let item1 = ItemActive {
        documento_fiscal_id: Set(documento.id),
        numero_item: Set(1),
        descricao: Set("Licença de software QuickNFe — plano anual".into()),
        codigo_ncm: Set(Some("85234990".into())),
        quantidade: Set(dec(1)),
        valor_unitario: Set(dec(1000)),
        valor_total_bruto: Set(dec(1000)),
        valor_desconto: Set(Some(dec(0))),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let item2 = ItemActive {
        documento_fiscal_id: Set(documento.id),
        numero_item: Set(2),
        descricao: Set("Suporte técnico especializado (12 meses)".into()),
        codigo_ncm: Set(Some("00000000".into())),
        quantidade: Set(dec(1)),
        valor_unitario: Set(dec(500)),
        valor_total_bruto: Set(dec(500)),
        valor_desconto: Set(Some(dec(0))),
        ..Default::default()
    }
    .insert(db)
    .await?;

    TributoActive {
        item_id: Set(item1.id),
        tipo_imposto: Set("ICMS".into()),
        cst_csosn: Set(Some("102".into())),
        base_calculo: Set(Some(dec(1000))),
        aliquota: Set(Some(dec_frac(0, 4))),
        valor_imposto: Set(Some(dec(0))),
        iss_retido: Set(Some(0)),
        aliquota_ibs: Set(Some(dec_frac(120, 4))),
        valor_ibs: Set(Some(dec(120))),
        aliquota_cbs: Set(Some(dec_frac(90, 4))),
        valor_cbs: Set(Some(dec(90))),
        ..Default::default()
    }
    .insert(db)
    .await?;

    TributoActive {
        item_id: Set(item1.id),
        tipo_imposto: Set("PIS".into()),
        cst_csosn: Set(Some("49".into())),
        base_calculo: Set(Some(dec(1000))),
        aliquota: Set(Some(dec_frac(65, 4))),
        valor_imposto: Set(Some(dec_frac(650, 2))),
        iss_retido: Set(Some(0)),
        ..Default::default()
    }
    .insert(db)
    .await?;

    TributoActive {
        item_id: Set(item1.id),
        tipo_imposto: Set("COFINS".into()),
        cst_csosn: Set(Some("49".into())),
        base_calculo: Set(Some(dec(1000))),
        aliquota: Set(Some(dec_frac(300, 4))),
        valor_imposto: Set(Some(dec(30))),
        iss_retido: Set(Some(0)),
        ..Default::default()
    }
    .insert(db)
    .await?;

    TributoActive {
        item_id: Set(item2.id),
        tipo_imposto: Set("ISS".into()),
        base_calculo: Set(Some(dec(500))),
        aliquota: Set(Some(dec_frac(500, 4))),
        valor_imposto: Set(Some(dec_frac(2500, 2))),
        iss_retido: Set(Some(1)),
        ..Default::default()
    }
    .insert(db)
    .await?;

    LogActive {
        documento_fiscal_id: Set(Some(documento.id)),
        lote_id: Set(Some(lote.id)),
        codigo_erro: Set("100".into()),
        mensagem_retorno: Set("Autorizado o uso da NF-e".into()),
        correcao_sugerida: Set(None),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(empresa)
}
