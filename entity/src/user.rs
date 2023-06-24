use crate::class::{email::Email, id::Id, password::HashedPassword, username::Username};
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id<Model>,
    #[sea_orm(unique)]
    pub username: Username,
    #[sea_orm(unique)]
    pub email: Email,
    pub password: HashedPassword,

    pub display_name: String,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub last_login: Option<DateTimeWithTimeZone>,
}

#[derive(Debug, Clone, Eq, PartialEq, DeriveIntoActiveModel, Serialize, Deserialize)]
pub struct NewUser {
    pub email: Email,
    pub username: Username,
    pub password: HashedPassword,
    pub display_name: String,
}

#[derive(Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        self.id = match self.id {
            ActiveValue::NotSet => ActiveValue::Set(Id::<Model>::create()),
            id => id,
        };
        self.password = match self.password {
            ActiveValue::Set(password) => ActiveValue::Set(password),
            pw => pw,
        };
        self.is_active = match self.is_active {
            ActiveValue::NotSet => ActiveValue::Set(false),
            set_or_unchanged => set_or_unchanged,
        };
        if self.is_changed() {
            let timestamp = chrono::Local::now().into();
            self.updated_at = ActiveValue::Set(timestamp);
            if insert {
                self.created_at = ActiveValue::Set(timestamp);
            }
        }
        Ok(self)
    }
}
