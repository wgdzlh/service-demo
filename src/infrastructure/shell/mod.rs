#![allow(dead_code)]

use tokio::time::Duration;

mod child;
pub use child::ChildWorker;
pub use child::ChildWorkerQueue;

use crate::app;

use super::config;
use child::ChildProc;

pub struct ChildWorkers {
    pub read_xls: ChildWorker,
}

impl ChildWorkers {
    pub async fn setup() -> app::Result<Self> {
        let py_config = config::peek_config()?.py.clone();
        let read_xls = ChildProc::setup(
            "python",
            Some(vec!["scripts/read_xls.py".to_owned()]),
            py_config.read_xls_workers,
            py_config.read_xls_inter_ms.map(Duration::from_millis),
            0,
        )
        .await?;
        Ok(Self { read_xls })
    }
}
