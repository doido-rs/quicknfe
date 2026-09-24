use doido::model::migration::{create_table, drop_table};
use doido::model::sea_orm::ConnectionTrait;
use doido::model::sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260924120000_create_documentos_fiscais_schema"
    }
}

async fn exec(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared(sql).await?;
    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_table(manager, "empresas", |t| {
            t.string("cpf_cnpj").not_null().unique_key();
            t.string("razao_social").not_null();
            t.string("inscricao_municipal");
            t.string("inscricao_estadual");
            t.integer("regime_tributario").not_null();
            t.string("codigo_municipio_ibge").not_null();
            t.string("uf").not_null();
            t.timestamp("criado_em")
                .not_null()
                .default(Expr::current_timestamp());
        })
        .await?;

        create_table(manager, "certificados_digitais", |t| {
            t.big_integer("empresa_id").not_null();
            t.string("tipo").not_null();
            t.string("caminho_arquivo");
            t.string("senha_encriptada").not_null();
            t.timestamp("data_validade").not_null();
            t.timestamp("criado_em")
                .not_null()
                .default(Expr::current_timestamp());
        })
        .await?;

        create_table(manager, "clientes", |t| {
            t.big_integer("empresa_id").not_null();
            t.string("cpf_cnpj").not_null();
            t.string("razao_social").not_null();
            t.string("inscricao_municipal");
            t.string("inscricao_estadual");
            t.string("logradouro");
            t.string("numero");
            t.string("bairro");
            t.string("codigo_municipio_ibge");
            t.string("uf");
            t.string("email");
            t.string("telefone");
        })
        .await?;

        create_table(manager, "lotes_transmissao", |t| {
            t.big_integer("empresa_id").not_null();
            t.big_integer("numero_lote").not_null();
            t.string("protocolo_recebimento");
            t.integer("ambiente").not_null();
            t.string("status_lote").not_null();
            t.timestamp("data_recebimento");
        })
        .await?;

        create_table(manager, "documentos_fiscais", |t| {
            t.big_integer("empresa_id").not_null();
            t.big_integer("cliente_id");
            t.big_integer("lote_id");
            t.string("modelo").not_null();
            t.string("serie").not_null();
            t.big_integer("numero_documento").not_null();
            t.string("chave_acesso").unique_key();
            t.string("codigo_verificacao");
            t.integer("ambiente").not_null();
            t.integer("tipo_emissao").not_null().default(1);
            t.string("status").not_null();
            t.timestamp("data_emissao").not_null();
            t.date("data_competencia");
            t.string("codigo_municipio_emissao").not_null();
            t.string("protocolo_autorizacao");
            t.decimal("valor_total_servicos_produtos")
                .not_null()
                .default(0);
            t.decimal("valor_liquido").not_null().default(0);
            t.text("xml_assinado");
            t.text("xml_processado");
        })
        .await?;

        create_table(manager, "itens_documento_fiscal", |t| {
            t.big_integer("documento_fiscal_id").not_null();
            t.integer("numero_item").not_null();
            t.text("descricao").not_null();
            t.string("codigo_ncm");
            t.string("codigo_tributacao_nacional");
            t.decimal("quantidade").not_null().default(1);
            t.decimal("valor_unitario").not_null().default(0);
            t.decimal("valor_total_bruto").not_null().default(0);
            t.decimal("valor_desconto").default(0);
        })
        .await?;

        create_table(manager, "tributos_item", |t| {
            t.big_integer("item_id").not_null();
            t.string("tipo_imposto").not_null();
            t.string("cst_csosn");
            t.decimal("base_calculo").default(0);
            t.decimal("aliquota").default(0);
            t.decimal("valor_imposto").default(0);
            t.boolean("iss_retido").default(false);
            t.string("classificacao_trib_ibscbs");
            t.decimal("aliquota_ibs").default(0);
            t.decimal("valor_ibs").default(0);
            t.decimal("aliquota_cbs").default(0);
            t.decimal("valor_cbs").default(0);
        })
        .await?;

        create_table(manager, "eventos_documento_fiscal", |t| {
            t.big_integer("documento_fiscal_id").not_null();
            t.string("tipo_evento").not_null();
            t.integer("sequencial_evento").not_null().default(1);
            t.string("protocolo_evento");
            t.text("descricao_motivo");
            t.timestamp("data_hora_evento").not_null();
            t.integer("ambiente").not_null();
            t.text("xml_evento");
        })
        .await?;

        create_table(manager, "logs_mensagens_retorno", |t| {
            t.big_integer("documento_fiscal_id");
            t.big_integer("lote_id");
            t.string("codigo_erro").not_null();
            t.text("mensagem_retorno").not_null();
            t.text("correcao_sugerida");
            t.timestamp("data_registro")
                .not_null()
                .default(Expr::current_timestamp());
        })
        .await?;

        exec(
            manager,
            "ALTER TABLE empresas
                ADD CONSTRAINT chk_empresas_cpf_cnpj_len
                CHECK (CHAR_LENGTH(cpf_cnpj) IN (11, 14))",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE certificados_digitais
                ADD CONSTRAINT chk_certificados_tipo
                CHECK (tipo IN ('A1', 'A3', 'PFX', 'PEM'))",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE clientes
                ADD CONSTRAINT chk_clientes_cpf_cnpj
                CHECK (CHAR_LENGTH(cpf_cnpj) IN (11, 14))",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE lotes_transmissao
                ADD CONSTRAINT chk_lotes_ambiente
                CHECK (ambiente IN (1, 2))",
        )
        .await?;

        exec(
            manager,
            "ALTER TABLE documentos_fiscais
                ADD CONSTRAINT chk_documentos_ambiente
                CHECK (ambiente IN (1, 2))",
        )
        .await?;

        exec(
            manager,
            "CREATE UNIQUE INDEX unq_empresa_doc
                ON documentos_fiscais (empresa_id, modelo, serie, numero_documento)",
        )
        .await?;

        exec(
            manager,
            "CREATE UNIQUE INDEX unq_item_doc
                ON itens_documento_fiscal (documento_fiscal_id, numero_item)",
        )
        .await?;

        exec(
            manager,
            "CREATE UNIQUE INDEX unq_evento_doc
                ON eventos_documento_fiscal (documento_fiscal_id, tipo_evento, sequencial_evento)",
        )
        .await?;

        for sql in [
            "ALTER TABLE certificados_digitais
                ADD CONSTRAINT fk_certificados_digitais_empresa_id
                FOREIGN KEY (empresa_id) REFERENCES empresas(id) ON DELETE CASCADE",
            "ALTER TABLE clientes
                ADD CONSTRAINT fk_clientes_empresa_id
                FOREIGN KEY (empresa_id) REFERENCES empresas(id) ON DELETE CASCADE",
            "ALTER TABLE lotes_transmissao
                ADD CONSTRAINT fk_lotes_transmissao_empresa_id
                FOREIGN KEY (empresa_id) REFERENCES empresas(id)",
            "ALTER TABLE documentos_fiscais
                ADD CONSTRAINT fk_documentos_fiscais_empresa_id
                FOREIGN KEY (empresa_id) REFERENCES empresas(id)",
            "ALTER TABLE documentos_fiscais
                ADD CONSTRAINT fk_documentos_fiscais_cliente_id
                FOREIGN KEY (cliente_id) REFERENCES clientes(id)",
            "ALTER TABLE documentos_fiscais
                ADD CONSTRAINT fk_documentos_fiscais_lote_id
                FOREIGN KEY (lote_id) REFERENCES lotes_transmissao(id)",
            "ALTER TABLE itens_documento_fiscal
                ADD CONSTRAINT fk_itens_documento_fiscal_documento_fiscal_id
                FOREIGN KEY (documento_fiscal_id) REFERENCES documentos_fiscais(id) ON DELETE CASCADE",
            "ALTER TABLE tributos_item
                ADD CONSTRAINT fk_tributos_item_item_id
                FOREIGN KEY (item_id) REFERENCES itens_documento_fiscal(id) ON DELETE CASCADE",
            "ALTER TABLE eventos_documento_fiscal
                ADD CONSTRAINT fk_eventos_documento_fiscal_documento_fiscal_id
                FOREIGN KEY (documento_fiscal_id) REFERENCES documentos_fiscais(id) ON DELETE CASCADE",
            "ALTER TABLE logs_mensagens_retorno
                ADD CONSTRAINT fk_logs_mensagens_retorno_documento_fiscal_id
                FOREIGN KEY (documento_fiscal_id) REFERENCES documentos_fiscais(id) ON DELETE CASCADE",
            "ALTER TABLE logs_mensagens_retorno
                ADD CONSTRAINT fk_logs_mensagens_retorno_lote_id
                FOREIGN KEY (lote_id) REFERENCES lotes_transmissao(id) ON DELETE CASCADE",
        ] {
            exec(manager, sql).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        drop_table(manager, "logs_mensagens_retorno").await?;
        drop_table(manager, "eventos_documento_fiscal").await?;
        drop_table(manager, "tributos_item").await?;
        drop_table(manager, "itens_documento_fiscal").await?;
        drop_table(manager, "documentos_fiscais").await?;
        drop_table(manager, "lotes_transmissao").await?;
        drop_table(manager, "clientes").await?;
        drop_table(manager, "certificados_digitais").await?;
        drop_table(manager, "empresas").await
    }
}
