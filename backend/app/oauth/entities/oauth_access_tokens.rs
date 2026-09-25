use doido::model::sea_orm;
use doido::model::sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "oauth_access_tokens")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub jti: String,
    pub user_id: i64,
    pub expires_at: DateTimeUtc,
    pub revoked: bool,
    pub refresh_jti: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
