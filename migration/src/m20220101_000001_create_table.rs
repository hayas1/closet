use entity::model::{health, user};
use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, EntityTrait, QueryTrait, Schema},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(manager.get_database_backend());
        manager.create_table(schema.create_table_from_entity(user::Entity)).await?;
        manager.create_table(schema.create_table_from_entity(health::Entity)).await?;

        let insert_ok =
            health::Entity::insert(health::ActiveModel { status: ActiveValue::Set("ok".into()) });
        manager.exec_stmt(insert_ok.into_query()).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let (drop_user_table, drop_health_table) = (
            manager.drop_table(Table::drop().table(user::Entity).to_owned()),
            manager.drop_table(Table::drop().table(health::Entity).to_owned()),
        );
        let (rut, rht) = tokio::join!(drop_user_table, drop_health_table);
        (rut?, rht?);
        Ok(())
    }
}
