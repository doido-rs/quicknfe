//! Montagem do XML da DPS (NFS-e Nacional) alinhada ao `layout_dps.xlsx`.

use super::dps_context::{
    cpf_cnpj_tag, decimal_xml, gerar_id_inf_dps, only_digits, ParteExterior, DpsContext,
};

use crate::models::documento_fiscal::Model as Documento;
use crate::services::api_error::ApiError;

const VER_APLIC: &str = "QuickNFe/0.1";
const DPS_VERSAO: &str = "1.01";

/// XML da DPS sem assinatura (rascunho). Prefer [`montar_dps_xml_context`] quando relações estão carregadas.
pub fn montar_dps_xml(doc: &Documento) -> String {
    let id = gerar_id_inf_dps(doc.id);
    let competencia = doc
        .data_competencia
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| doc.data_emissao.format("%Y-%m-%d").to_string());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<DPS xmlns="http://www.gov.br/nfse" versao="{DPS_VERSAO}">
  <infDPS Id="{id}">
    <tpAmb>{ambiente}</tpAmb>
    <dhEmi>{emissao}</dhEmi>
    <verAplic>{VER_APLIC}</verAplic>
    <serie>{serie}</serie>
    <nDPS>{numero}</nDPS>
    <dCompet>{competencia}</dCompet>
    <tpEmit>1</tpEmit>
    <cLocEmi>0000000</cLocEmi>
    <vServ>{valor}</vServ>
  </infDPS>
</DPS>"#,
        ambiente = doc.ambiente,
        emissao = doc.data_emissao.to_rfc3339(),
        serie = xml_escape(&doc.serie),
        numero = doc.numero_documento,
        valor = decimal_xml(&doc.valor_total_servicos_produtos),
    )
}

pub fn montar_dps_xml_context(ctx: &DpsContext) -> Result<String, ApiError> {
    let doc = &ctx.doc;
    let id = gerar_id_inf_dps(doc.id);
    let competencia = doc
        .data_competencia
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| doc.data_emissao.format("%Y-%m-%d").to_string());

    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str(&format!(
        r#"<DPS xmlns="http://www.gov.br/nfse" versao="{DPS_VERSAO}">"#
    ));
    xml.push('\n');
    xml.push_str(&format!(r#"  <infDPS Id="{id}">"#));
    xml.push('\n');
    append_identificacao(&mut xml, ctx, &competencia);
    append_prestador(&mut xml, ctx)?;
    append_tomador(&mut xml, ctx);
    append_intermediario(&mut xml, ctx);
    append_servico(&mut xml, ctx);
    append_valores(&mut xml, ctx);
    xml.push_str("  </infDPS>\n</DPS>\n");
    Ok(xml)
}

fn append_identificacao(xml: &mut String, ctx: &DpsContext, competencia: &str) {
    let doc = &ctx.doc;
    push_elem(xml, 4, "tpAmb", &doc.ambiente.to_string());
    push_elem(xml, 4, "dhEmi", &doc.data_emissao.to_rfc3339());
    push_elem(xml, 4, "verAplic", VER_APLIC);
    push_elem(xml, 4, "serie", &doc.serie);
    push_elem(xml, 4, "nDPS", &doc.numero_documento.to_string());
    push_elem(xml, 4, "dCompet", competencia);
    push_elem(xml, 4, "tpEmit", "1");
    push_elem(xml, 4, "tpEmis", &doc.tipo_emissao.to_string());
    push_elem(
        xml,
        4,
        "cLocEmi",
        &pad_ibge7(&ctx.municipio_emissao.codigo_ibge),
    );
}

fn append_prestador(xml: &mut String, ctx: &DpsContext) -> Result<(), ApiError> {
    let p = &ctx.prestador;
    xml.push_str("    <prest>\n");
    let (tag, insc) = cpf_cnpj_tag(&p.cpf_cnpj);
    push_elem(xml, 6, tag, &insc);
    if let Some(im) = p.inscricao_municipal.as_ref().filter(|s| !s.trim().is_empty()) {
        push_elem(xml, 6, "IM", im);
    }
    push_elem(xml, 6, "xNome", &xml_escape(&p.razao_social));
    xml.push_str("      <end>\n");
    xml.push_str("        <endNac>\n");
    push_elem(
        xml,
        10,
        "cMun",
        &pad_ibge7(&ctx.municipio_prestador.codigo_ibge),
    );
    push_elem(xml, 10, "CEP", "00000000");
    xml.push_str("        </endNac>\n");
    push_elem(xml, 8, "xLgr", "NAO INFORMADO");
    push_elem(xml, 8, "nro", "S/N");
    push_elem(xml, 8, "xBairro", "NAO INFORMADO");
    xml.push_str("      </end>\n");
    xml.push_str("      <regTrib>\n");
    push_elem(xml, 8, "opSimpNac", &map_op_simp_nac(&p.regime_tributario));
    push_elem(xml, 8, "regEspTrib", &ctx.doc.reg_esp_trib_issqn);
    xml.push_str("      </regTrib>\n");
    xml.push_str("    </prest>\n");
    Ok(())
}

fn map_op_simp_nac(regime: &str) -> String {
    match regime {
        "MICROEMPREENDEDOR_INDIVIDUAL" => "2".into(),
        "SIMPLES_NACIONAL" | "SIMPLES_NACIONAL_EXCESSO_SUBLIMITE" => "1".into(),
        _ => "3".into(),
    }
}

fn append_tomador(xml: &mut String, ctx: &DpsContext) {
    if ctx.doc.tomador_localidade == "nao_informado" {
        return;
    }
    if ctx.doc.tomador_localidade == "exterior" {
        if let Some(ext) = &ctx.tomador_exterior {
            append_parte_exterior(xml, "toma", ext);
        }
        return;
    }
    if let Some(t) = &ctx.tomador {
        append_parte_brasil(
            xml,
            "toma",
            &t.cpf_cnpj,
            &t.razao_social,
            t.inscricao_municipal.as_deref(),
            t.email.as_deref(),
            t.telefone.as_deref(),
            ctx.tomador_municipio.as_ref(),
        );
    }
}

fn append_intermediario(xml: &mut String, ctx: &DpsContext) {
    if ctx.doc.intermediario_localidade == "nao_informado" {
        return;
    }
    if ctx.doc.intermediario_localidade == "exterior" {
        if let Some(ext) = &ctx.intermediario_exterior {
            append_parte_exterior(xml, "interm", ext);
        }
        return;
    }
    if let Some(i) = &ctx.intermediario {
        append_parte_brasil(
            xml,
            "interm",
            &i.cpf_cnpj,
            &i.razao_social,
            i.inscricao_municipal.as_deref(),
            i.email.as_deref(),
            i.telefone.as_deref(),
            None,
        );
    }
}

fn append_parte_brasil(
    xml: &mut String,
    grupo: &str,
    cpf_cnpj: &str,
    razao: &str,
    im: Option<&str>,
    email: Option<&str>,
    fone: Option<&str>,
    municipio: Option<&crate::models::municipio::Model>,
) {
    xml.push_str(&format!("    <{grupo}>\n"));
    let (tag, insc) = cpf_cnpj_tag(cpf_cnpj);
    push_elem(xml, 6, tag, &insc);
    if let Some(im) = im.filter(|s| !s.trim().is_empty()) {
        push_elem(xml, 6, "IM", im);
    }
    push_elem(xml, 6, "xNome", &xml_escape(razao));
    if municipio.is_some() {
        xml.push_str("      <end>\n");
        xml.push_str("        <endNac>\n");
        if let Some(m) = municipio {
            push_elem(
                xml,
                10,
                "cMun",
                &pad_ibge7(&m.codigo_ibge),
            );
        }
        push_elem(xml, 10, "CEP", "00000000");
        xml.push_str("        </endNac>\n");
        push_elem(xml, 8, "xLgr", "NAO INFORMADO");
        push_elem(xml, 8, "nro", "S/N");
        push_elem(xml, 8, "xBairro", "NAO INFORMADO");
        xml.push_str("      </end>\n");
    }
    if let Some(f) = fone.filter(|s| !s.trim().is_empty()) {
        push_elem(xml, 6, "fone", &only_digits(f));
    }
    if let Some(e) = email.filter(|s| !s.trim().is_empty()) {
        push_elem(xml, 6, "email", &xml_escape(e));
    }
    xml.push_str(&format!("    </{grupo}>\n"));
}

fn append_parte_exterior(xml: &mut String, grupo: &str, ext: &ParteExterior) {
    xml.push_str(&format!("    <{grupo}>\n"));
    push_elem(xml, 6, "NIF", "000000000");
    push_elem(xml, 6, "cNaoNIF", "2");
    push_elem(xml, 6, "xNome", &xml_escape(&ext.razao_social));
    xml.push_str("      <end>\n");
    xml.push_str("        <endExt>\n");
    push_elem(xml, 10, "cPais", &ext.pais.chars().take(2).collect::<String>());
    push_elem(xml, 10, "cEndPost", &xml_escape(&ext.codigo_postal));
    push_elem(xml, 10, "xCidade", &xml_escape(&ext.cidade));
    push_elem(xml, 10, "xEstProvReg", &xml_escape(&ext.estado_regiao));
    xml.push_str("        </endExt>\n");
    push_elem(xml, 8, "xLgr", &xml_escape(&ext.logradouro));
    push_elem(xml, 8, "nro", &xml_escape(&ext.numero));
    if let Some(c) = ext.complemento.as_ref().filter(|s| !s.trim().is_empty()) {
        push_elem(xml, 8, "xCpl", &xml_escape(c));
    }
    push_elem(xml, 8, "xBairro", &xml_escape(&ext.bairro));
    xml.push_str("      </end>\n");
    if let Some(f) = ext.telefone.as_ref().filter(|s| !s.trim().is_empty()) {
        push_elem(xml, 6, "fone", &only_digits(f));
    }
    if let Some(e) = ext.email.as_ref().filter(|s| !s.trim().is_empty()) {
        push_elem(xml, 6, "email", &xml_escape(e));
    }
    xml.push_str(&format!("    </{grupo}>\n"));
}

fn append_servico(xml: &mut String, ctx: &DpsContext) {
    xml.push_str("    <serv>\n");
    xml.push_str("      <locPrest>\n");
    push_elem(
        xml,
        8,
        "cLocPrestacao",
        &pad_ibge7(&ctx.municipio_prestacao.codigo_ibge),
    );
    xml.push_str("      </locPrest>\n");
    xml.push_str("      <cServ>\n");
    let (c_trib, x_desc) = servico_from_itens(&ctx.itens, &ctx.doc);
    push_elem(xml, 8, "cTribNac", &c_trib);
    push_elem(xml, 8, "xDescServ", &xml_escape(&x_desc));
    xml.push_str("      </cServ>\n");
    if let Some(info) = info_complementar_text(ctx) {
        push_elem(xml, 6, "infoCompl", &xml_escape(&info));
    }
    xml.push_str("    </serv>\n");
}

fn info_complementar_text(ctx: &DpsContext) -> Option<String> {
    let doc = &ctx.doc;
    let mut parts = Vec::new();
    if let Some(s) = doc.informacoes_complementares.as_ref().filter(|s| !s.trim().is_empty()) {
        parts.push(s.trim());
    }
    if let Some(s) = doc
        .numero_doc_responsabilidade_tecnica
        .as_ref()
        .filter(|s| !s.trim().is_empty())
    {
        parts.push(s.trim());
    }
    if let Some(s) = doc.documento_referencia.as_ref().filter(|s| !s.trim().is_empty()) {
        parts.push(s.trim());
    }
    if let Some(s) = doc.numero_pedido_ordem_b2b.as_ref().filter(|s| !s.trim().is_empty()) {
        parts.push(s.trim());
    }
    if parts.is_empty() {
        return None;
    }
    let joined = parts.join(" — ");
    Some(if joined.len() > 2000 {
        joined[..2000].to_string()
    } else {
        joined
    })
}

fn servico_from_itens(
    itens: &[crate::models::item_documento_fiscal::Model],
    doc: &Documento,
) -> (String, String) {
    if itens.is_empty() {
        return (
            "010101".into(),
            "Serviço prestado".into(),
        );
    }
    let mut desc_parts = Vec::new();
    let mut c_trib = itens[0]
        .codigo_tributacao_nacional
        .clone()
        .unwrap_or_else(|| "010101".into());
    for item in itens {
        if let Some(c) = item.codigo_tributacao_nacional.as_ref() {
            if !c.trim().is_empty() {
                c_trib = only_digits(c);
                if c_trib.len() > 6 {
                    c_trib = c_trib[..6].to_string();
                }
                while c_trib.len() < 6 {
                    c_trib.insert(0, '0');
                }
            }
        }
        let d = item
            .descricao_servico
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(item.descricao.as_str());
        desc_parts.push(d.trim());
    }
    let mut x_desc = desc_parts.join(" | ");
    if x_desc.len() > 1000 {
        x_desc.truncate(1000);
    }
    if x_desc.is_empty() {
        x_desc = format!("Nota fiscal {}", doc.numero_documento);
    }
    (c_trib, x_desc)
}

fn append_valores(xml: &mut String, ctx: &DpsContext) {
    let doc = &ctx.doc;
    xml.push_str("    <valores>\n");
    xml.push_str("      <vServPrest>\n");
    push_elem(
        xml,
        8,
        "vServ",
        &decimal_xml(&doc.valor_total_servicos_produtos),
    );
    if let Some(v) = doc.valor_recebido_intermediario {
        if v > rust_decimal::Decimal::ZERO {
            push_elem(xml, 8, "vReceb", &decimal_xml(&v));
        }
    }
    xml.push_str("      </vServPrest>\n");
    if doc.desconto_incondicionado > rust_decimal::Decimal::ZERO
        || doc.desconto_condicionado > rust_decimal::Decimal::ZERO
    {
        xml.push_str("      <vDescCondIncond>\n");
        if doc.desconto_incondicionado > rust_decimal::Decimal::ZERO {
            push_elem(xml, 8, "vDescIncond", &decimal_xml(&doc.desconto_incondicionado));
        }
        if doc.desconto_condicionado > rust_decimal::Decimal::ZERO {
            push_elem(xml, 8, "vDescCond", &decimal_xml(&doc.desconto_condicionado));
        }
        xml.push_str("      </vDescCondIncond>\n");
    }
    xml.push_str("      <trib>\n");
    xml.push_str("        <tribMun>\n");
    push_elem(xml, 10, "tribISSQN", &doc.trib_issqn);
    push_elem(
        xml,
        10,
        "tpRetISSQN",
        if doc.issqn_retencao == 1 { "2" } else { "1" },
    );
    if doc.issqn_exigibilidade_suspensa == 1 {
        push_elem(xml, 10, "tpImunidade", "0");
    }
    if doc.issqn_beneficio_municipal == 1 {
        push_elem(xml, 10, "nBM", "0");
    }
    if let Some(v) = doc.issqn_aliquota {
        push_elem(xml, 10, "pAliq", &decimal_xml(&v));
    }
    if let Some(v) = doc.issqn_base_calculo {
        push_elem(xml, 10, "vBC", &decimal_xml(&v));
    }
    if let Some(v) = doc.issqn_valor {
        push_elem(xml, 10, "vISSQN", &decimal_xml(&v));
    }
    xml.push_str("        </tribMun>\n");
    xml.push_str("        <tribFed>\n");
    xml.push_str("          <piscofins>\n");
    push_elem(xml, 12, "CST", &doc.cst_pis_cofins);
    push_elem(xml, 12, "tpRetPisCofins", &doc.tp_ret_pis_cofins_csll);
    xml.push_str("          </piscofins>\n");
    if let Some(v) = doc.valor_contrib_previdenciaria_retida {
        push_elem(xml, 10, "vRetCP", &decimal_xml(&v));
    }
    if let Some(v) = doc.valor_irrf {
        push_elem(xml, 10, "vRetIRRF", &decimal_xml(&v));
    }
    if let Some(v) = doc.valor_contrib_sociais_retidas {
        push_elem(xml, 10, "vRetCSLL", &decimal_xml(&v));
    }
    xml.push_str("        </tribFed>\n");
    xml.push_str("      </trib>\n");
    xml.push_str("    </valores>\n");
}

fn pad_ibge7(code: &str) -> String {
    let digits = only_digits(code);
    format!("{:0>7}", digits)
}

fn push_elem(xml: &mut String, indent: usize, tag: &str, value: &str) {
    let pad = " ".repeat(indent);
    xml.push_str(&format!("{pad}<{tag}>{value}</{tag}>\n"));
}

pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub mod coverage {
    use super::super::dps_layout::{load_infdps_leiaute, mandatory_infdps_fields};
    use std::collections::HashSet;
    use std::path::Path;

    /// Tags de elemento (`campo`) que o builder emite hoje (fases 3A–3H).
    pub fn emitted_tags() -> HashSet<&'static str> {
        [
            "tpAmb", "dhEmi", "verAplic", "serie", "nDPS", "dCompet", "tpEmit", "tpEmis", "cLocEmi",
            "CNPJ", "CPF", "IM", "xNome", "cMun", "CEP", "xLgr", "nro", "xBairro", "opSimpNac",
            "regEspTrib", "NIF", "cNaoNIF", "fone", "email", "cLocPrestacao", "cTribNac",
            "xDescServ", "vServ", "vReceb", "vDescIncond", "vDescCond", "tribISSQN", "tpRetISSQN",
            "pAliq", "vBC", "vISSQN", "tpImunidade", "nBM", "CST", "tpRetPisCofins", "vRetCP",
            "vRetIRRF", "vRetCSLL",
            "infoCompl", "cPais", "cEndPost", "xCidade", "xEstProvReg", "xCpl",
        ]
        .into_iter()
        .collect()
    }

    pub fn coverage_report(repo_root: &Path) -> Result<(usize, usize, Vec<String>), String> {
        let rows = load_infdps_leiaute(repo_root)?;
        let mandatory = mandatory_infdps_fields(&rows);
        let emitted = emitted_tags();
        let missing: Vec<String> = mandatory
            .iter()
            .filter(|r| !emitted.contains(r.campo.as_str()))
            .map(|r| format!("{} ({})", r.campo, r.xml_path))
            .collect();
        Ok((mandatory.len() - missing.len(), mandatory.len(), missing))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use doido::model::sea_orm::prelude::Decimal;
    use std::path::PathBuf;

    fn sample_doc() -> Documento {
        Documento {
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
        }
    }

    #[test]
    fn montar_xml_minimo_contem_inf_dps() {
        let xml = montar_dps_xml(&sample_doc());
        assert!(xml.contains("<infDPS Id="));
        assert!(xml.contains("<tpAmb>2</tpAmb>"));
    }

    #[test]
    fn layout_cobertura_parcial() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let (hit, total, missing) = coverage::coverage_report(&root).expect("layout");
        assert!(hit > 0, "nenhum campo obrigatório coberto");
        assert!(
            total > hit,
            "cobertura completa inesperada neste teste: {hit}/{total}"
        );
        // Baseline documentado (Phase 4): aumentar `min_ratio` conforme o builder evolui.
        let min_ratio = 0.22_f64;
        let ratio = hit as f64 / total as f64;
        assert!(
            ratio >= min_ratio,
            "cobertura {hit}/{total} ({ratio:.1}%) abaixo do mínimo {min_ratio:.0}% — faltam: {}",
            missing.len()
        );
    }

    #[test]
    fn assinar_insere_signature() {
        let raw = r#"<?xml version="1.0"?><DPS><infDPS Id="DPS1"/></DPS>"#;
        let signed = super::super::assinatura::assinar_dps_xml_rascunho(raw, None, None).unwrap();
        assert!(signed.contains("<Signature"));
    }

    #[test]
    fn montar_dps_contexto_inclui_grupos_principais() {
        use super::super::dps_context::DpsContext;
        use crate::models::empresa::Model as Empresa;
        use crate::models::item_documento_fiscal::Model as ItemDoc;
        use crate::models::municipio::Model as Municipio;
        use doido::model::sea_orm::prelude::Decimal;

        let municipio = Municipio {
            id: 1,
            codigo_ibge: "3304557".into(),
            nome: "Rio de Janeiro".into(),
            uf: "RJ".into(),
        };
        let ctx = DpsContext {
            doc: sample_doc(),
            prestador: Empresa {
                id: 1,
                company_id: 1,
                cpf_cnpj: "12345678000199".into(),
                razao_social: "Prestador LTDA".into(),
                inscricao_municipal: Some("123".into()),
                inscricao_estadual: None,
                regime_tributario: "SIMPLES_NACIONAL".into(),
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
                descricao_servico: Some("Consultoria".into()),
                codigo_ncm: None,
                codigo_tributacao_nacional: Some("010101".into()),
                cna_id: None,
                quantidade: Decimal::from(1),
                valor_unitario: Decimal::from(100),
                valor_total_bruto: Decimal::from(100),
                valor_desconto: None,
            }],
        };
        super::super::rn_validacao::validar_dps_contexto(&ctx).expect("rn");
        let xml = montar_dps_xml_context(&ctx).expect("xml");
        assert!(xml.contains("<prest>"));
        assert!(xml.contains("<valores>"));
        assert!(xml.contains("<cLocEmi>3304557</cLocEmi>"));
    }

    #[test]
    fn montar_dps_golden_fixture_principais_tags() {
        use super::super::dps_context::DpsContext;
        use crate::models::empresa::Model as Empresa;
        use crate::models::item_documento_fiscal::Model as ItemDoc;
        use crate::models::municipio::Model as Municipio;
        use doido::model::sea_orm::prelude::Decimal;

        let municipio = Municipio {
            id: 1,
            codigo_ibge: "3304557".into(),
            nome: "Rio de Janeiro".into(),
            uf: "RJ".into(),
        };
        let ctx = DpsContext {
            doc: sample_doc(),
            prestador: Empresa {
                id: 1,
                company_id: 1,
                cpf_cnpj: "12345678000199".into(),
                razao_social: "Prestador LTDA".into(),
                inscricao_municipal: Some("123".into()),
                inscricao_estadual: None,
                regime_tributario: "SIMPLES_NACIONAL".into(),
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
                codigo_ncm: None,
                codigo_tributacao_nacional: Some("010101".into()),
                cna_id: None,
                quantidade: Decimal::from(1),
                valor_unitario: Decimal::from(100),
                valor_total_bruto: Decimal::from(100),
                valor_desconto: None,
            }],
        };
        let xml = montar_dps_xml_context(&ctx).expect("xml");
        for tag in ["<tpEmis>", "<cTribNac>", "<xDescServ>", "<tribISSQN>"] {
            assert!(xml.contains(tag), "missing {tag}");
        }
    }
}
