use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::class::status::Status;

#[derive(Debug, Clone, Eq, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "health")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub status: Status,
}

#[derive(Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}
