mod todo;
pub use todo::TodoStore;

mod post;
pub use post::PostStore;

use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database};
use tracing::log::LevelFilter;

use crate::app;

use super::config;

pub struct Db {
    pub todo: TodoStore,
    pub post: PostStore,
}

impl Db {
    pub async fn setup() -> app::Result<Self> {
        let mut opt;
        {
            let cc = config::C.lock().unwrap();
            let cc = cc.borrow();
            let db_conf = &cc.db;
            opt = ConnectOptions::new(db_conf.url.clone());
            if !db_conf.log_mode {
                opt.sqlx_logging_level(LevelFilter::Trace);
            }
        }

        let conn = Database::connect(opt)
            .await
            .expect("Database connection failed");

        Migrator::up(&conn, None).await?;
        Ok(Self {
            todo: todo::get_todo_store(),
            post: post::get_post_store(&conn),
        })
    }
}
