use crate::{
    infrastructure::{persistence, shell},
    interface::route,
};

#[tokio::main]
pub async fn run() -> super::Result<()> {
    let db = persistence::Db::setup().await?;
    let child_workers = shell::ChildWorkers::setup().await?;

    route::serve(db, child_workers).await
}
