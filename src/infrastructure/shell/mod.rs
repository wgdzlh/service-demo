#![allow(dead_code)]

use std::sync::Arc;

use tokio::time::Duration;

mod child;
pub use child::ChildProc;

use crate::app;

use super::config;

pub type ChildWorker = Arc<ChildProc>;

pub struct ChildWorkers {
    pub read_xls: ChildWorker,
}

impl ChildWorkers {
    pub async fn setup() -> app::Result<Self> {
        let py_config = config::peek_config()?.py.clone();
        let read_xls = Arc::new(
            ChildProc::setup(
                "python",
                Some(vec!["scripts/read_xls.py".to_owned()]),
                py_config.read_xls_workers,
                py_config.read_xls_inter_ms.map(Duration::from_millis),
            )
            .await?,
        );
        Ok(Self { read_xls })
    }
}
