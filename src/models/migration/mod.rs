use sea_orm_migration::prelude::*;

mod m20241010_000000_create_table;
mod m20241201_000000_add_userinfo;
mod m20250709_000000_create_premission_role;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241010_000000_create_table::Migration),
            Box::new(m20241201_000000_add_userinfo::Migration),
            Box::new(m20250709_000000_create_premission_role::Migration),
        ]
    }
}
