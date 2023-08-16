use migration::{Migrator, MigratorTrait};
use sea_orm::Database;

use crate::{infrastructure::persistence, interface::route};

#[tokio::main]
pub async fn run() -> super::Result<()> {
    let db_url = "postgres://postgres:bjsh@192.168.0.221:20397/test?sslmode=disable";
    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");

    Migrator::up(&conn, None).await?;
    let db = persistence::Db::new(&conn);
    route::serve(db).await?;
    Ok(())
}
