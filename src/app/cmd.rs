use crate::{infrastructure::persistence, interface::route};

#[tokio::main]
pub async fn run() -> super::Result<()> {
    let db = persistence::Db::setup().await?;
    route::serve(db).await?;
    Ok(())
}
