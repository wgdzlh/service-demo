pub use sea_orm_migration::prelude::*;

mod m20230816_032228_create_post_table;
mod m20230816_032636_add_post_fields;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230816_032228_create_post_table::Migration),
            Box::new(m20230816_032636_add_post_fields::Migration),
        ]
    }
}
