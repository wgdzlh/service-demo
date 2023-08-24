use crate::{
    infrastructure::{config, persistence, shell},
    interface::route,
};

#[tokio::main]
pub async fn run() -> super::Result<()> {
    if config::get_config()?.server.run_local {
        tokio::spawn(config::local_conf_watch());
    }
    let db = persistence::Db::setup().await?;
    let child_workers = shell::ChildWorkers::setup().await?;

    route::serve(db, child_workers).await
}
