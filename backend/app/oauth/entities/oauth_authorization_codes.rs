use doido::model::sea_orm;
use doido::model::sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "oauth_authorization_codes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub code: String,
    pub client_id: String,
    pub user_id: i64,
    pub code_challenge: String,
    pub redirect_uri: String,
    #[sea_orm(column_type = "Text")]
    pub session_cookie: String,
    pub expires_at: DateTimeUtc,
    pub used: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
