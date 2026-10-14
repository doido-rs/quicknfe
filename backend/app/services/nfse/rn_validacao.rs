//! Regras de negócio (RN) aplicáveis antes de montar/transmitir a DPS.

use doido::model::sea_orm::prelude::Decimal;

use super::dps_context::DpsContext;
use super::dps_presenca;
use super::dps_rn;
use super::dps_rn_engine;
use crate::services::api_error::ApiError;

/// Valida documento + contexto carregado (espelha RN críticas do leiaute / formulário).
pub fn validar_dps_contexto(ctx: &DpsContext) -> Result<(), ApiError> {
    let doc = &ctx.doc;
    if doc.valor_total_servicos_produtos <= Decimal::ZERO {
        return Err(ApiError::Validation(
            "RN: valor da operação (vServ) deve ser positivo".into(),
        ));
    }
    if doc.serie_numero_automatico == 0 && doc.serie.trim().is_empty() {
        return Err(ApiError::Validation("RN: série da DPS é obrigatória".into()));
    }
    if doc.serie_numero_automatico == 0 && doc.numero_documento <= 0 {
        return Err(ApiError::Validation(
            "RN: número da DPS deve ser positivo".into(),
        ));
    }
    if ctx.municipio_emissao.codigo_ibge.trim().is_empty() {
        return Err(ApiError::Validation(
            "RN: município de emissão (cLocEmi) inválido".into(),
        ));
    }
    if ctx.municipio_prestacao.codigo_ibge.trim().is_empty() {
        return Err(ApiError::Validation(
            "RN: município de prestação (cLocPrestacao) inválido".into(),
        ));
    }
    if doc.tomador_localidade == "brasil" && doc.cliente_id.is_none() {
        return Err(ApiError::Validation(
            "RN: tomador Brasil exige cliente_id".into(),
        ));
    }
    if doc.tomador_localidade == "exterior" && ctx.tomador_exterior.is_none() {
        return Err(ApiError::Validation(
            "RN: tomador exterior exige tomador_exterior".into(),
        ));
    }
    if doc.intermediario_localidade == "brasil" && doc.intermediario_id.is_none() {
        return Err(ApiError::Validation(
            "RN: intermediário Brasil exige intermediario_id".into(),
        ));
    }
    if doc.intermediario_localidade == "exterior" && ctx.intermediario_exterior.is_none() {
        return Err(ApiError::Validation(
            "RN: intermediário exterior exige intermediario_exterior".into(),
        ));
    }
    if doc.ambiente != 1 && doc.ambiente != 2 {
        return Err(ApiError::Validation(
            "RN: tpAmb deve ser 1 (produção) ou 2 (homologação)".into(),
        ));
    }
    if ctx.itens.is_empty() && doc.valor_total_servicos_produtos > Decimal::ZERO {
        return Err(ApiError::Validation(
            "RN: nota com valor da operação exige ao menos um item de serviço".into(),
        ));
    }
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    dps_rn::load_rn_dps_rows(&root)
        .map_err(|e| ApiError::Internal(format!("RN layout indisponível: {e}")))?;
    dps_presenca::validar_presenca_repo(ctx)?;
    dps_rn_engine::validar_rn_csv_incremental(ctx)?;
    Ok(())
}

/// Garante que o workbook RN está presente (sanity para CI / deploy).
pub fn validar_rn_workbook_disponivel() -> Result<(), ApiError> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    dps_rn::load_rn_dps_rows(&root).map_err(|e| ApiError::Internal(format!("RN layout: {e}")))?;
    Ok(())
}

#[cfg(test)]
pub fn minimal_dps_context() -> DpsContext {
    use chrono::Utc;
        use crate::models::documento_fiscal::Model as Documento;
        use crate::models::empresa::Model as Empresa;
        use crate::models::item_documento_fiscal::Model as ItemDoc;
        use crate::models::municipio::Model as Municipio;

        let municipio = Municipio {
            id: 1,
            codigo_ibge: "3304557".into(),
            nome: "Rio de Janeiro".into(),
            uf: "RJ".into(),
        };
        DpsContext {
            doc: Documento {
                id: 1,
                empresa_id: 1,
                cliente_id: None,
                intermediario_id: None,
                tomador_localidade: "nao_informado".into(),
                intermediario_localidade: "nao_informado".into(),
                tomador_exterior: None,
                intermediario_exterior: None,
                lote_id: None,
                modelo: "NFSe".into(),
                serie: "AUTO".into(),
                numero_documento: 1,
                chave_acesso: None,
                codigo_verificacao: None,
                ambiente: 2,
                tipo_emissao: 1,
                status: "novo".into(),
                data_emissao: Utc::now(),
                data_competencia: None,
                serie_numero_automatico: 1,
                municipio_emissao_id: 1,
                municipio_prestacao_id: 1,
                protocolo_autorizacao: None,
                valor_total_servicos_produtos: Decimal::from(100),
                valor_liquido: Decimal::from(100),
                valor_recebido_intermediario: None,
                desconto_incondicionado: Decimal::ZERO,
                desconto_condicionado: Decimal::ZERO,
                trib_issqn: "1".into(),
                reg_esp_trib_issqn: "0".into(),
                issqn_exigibilidade_suspensa: 0,
                issqn_retencao: 0,
                issqn_beneficio_municipal: 0,
                issqn_deducao_reducao_bc: 0,
                issqn_aliquota: None,
                issqn_base_calculo: None,
                issqn_valor: None,
                cst_pis_cofins: "00".into(),
                tp_ret_pis_cofins_csll: "0".into(),
                valor_irrf: None,
                valor_contrib_sociais_retidas: None,
                valor_contrib_previdenciaria_retida: None,
                xml_assinado: None,
                xml_processado: None,
                numero_doc_responsabilidade_tecnica: None,
                documento_referencia: None,
                informacoes_complementares: None,
                numero_pedido_ordem_b2b: None,
            },
            prestador: Empresa {
                id: 1,
                company_id: 1,
                cpf_cnpj: "12345678000199".into(),
                razao_social: "Prestador Teste".into(),
                inscricao_municipal: None,
                inscricao_estadual: None,
                regime_tributario: "REGIME_NORMAL".into(),
                municipio_id: 1,
                criado_em: Utc::now(),
            },
            municipio_emissao: municipio.clone(),
            municipio_prestacao: municipio.clone(),
            municipio_prestador: municipio,
            tomador: None,
            tomador_municipio: None,
            tomador_exterior: None,
            intermediario: None,
            intermediario_municipio: None,
            intermediario_exterior: None,
            itens: vec![ItemDoc {
                id: 1,
                documento_fiscal_id: 1,
                numero_item: 1,
                descricao: "Serviço".into(),
                descricao_servico: Some("Serviço".into()),
                codigo_tributacao_nacional: Some("010101".into()),
                codigo_ncm: None,
                cna_id: None,
                quantidade: Decimal::from(1),
                valor_unitario: Decimal::from(100),
                valor_total_bruto: Decimal::from(100),
                valor_desconto: None,
            }],
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rn_aceita_contexto_minimo() {
        validar_dps_contexto(&minimal_dps_context()).expect("ok");
    }

    #[test]
    fn rn_rejeita_valor_zero() {
        let mut ctx = minimal_dps_context();
        ctx.doc.valor_total_servicos_produtos = Decimal::ZERO;
        assert!(validar_dps_contexto(&ctx).is_err());
    }

    #[test]
    fn rn_workbook_carrega() {
        validar_rn_workbook_disponivel().expect("layout rn");
    }
}
