use std::collections::HashMap;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as Lock;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Semaphore;
use tokio::time::{Duration, Instant, MissedTickBehavior};

use crate::app::utils::{remove_enter, trim_end_inplace};
use crate::infrastructure::config;
use crate::repository::Error::{EmptyRet, RunSubCmdError, SubmitTimeout};
use crate::{app::log::*, repository::Result};

const BUFF_SIZE: usize = 65536;
const CMD_SUBMIT_TIMEOUT: Duration = Duration::from_secs(5);
const BOUND_WORKER_TIMEOUT: Duration = Duration::from_secs(660);
const BOUND_WORKER_CHECK_INTER: Duration = Duration::from_secs(60);

const ERROR_SIG_CHAR: u8 = b'!'; // special first char to indicate child worker has output error info

pub type ChildWorker = Arc<ChildProc>;
pub type ChildWorkerQueue = Arc<ChildProcQueue>;

pub struct ChildProc {
    cmd: Arc<Cmd>,
    workers: Mutex<Vec<Worker>>,
    workers_sem: Semaphore,
    concurrent: usize,
}

pub struct ChildProcQueue {
    cmd: Arc<Cmd>,
    workers: flume::Receiver<SharedWorker>,
    recycler: flume::Sender<SharedWorker>,
    worker_map: Mutex<HashMap<String, SharedWorker>>,
    worker_cond: Mutex<HashMap<String, Cond>>,
}

type SharedWorker = Arc<Lock<Worker>>;
type Cond = Arc<Lock<()>>;
// type Cond = (flume::Sender<()>, flume::Receiver<()>);

struct Cmd {
    bin: String,
    args: Option<Vec<String>>,
}

struct OnceWorker {
    cmd: Arc<Cmd>,
    proc: Child,
    stdin: ChildStdin,
}

struct Worker {
    cmd: Arc<Cmd>,
    proc: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    ts: Instant,
}

impl OnceWorker {
    fn init(cmd: &Arc<Cmd>) -> Result<Self> {
        let cmd = cmd.clone();
        let mut child_def = Command::new(&cmd.bin);
        if let Some(args) = &cmd.args {
            child_def.args(args);
        }
        let mut proc = child_def
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let stdin = proc.stdin.take().expect("Failed to open stdin");
        Ok(Self { cmd, proc, stdin })
    }

    async fn run(mut self, input: String) -> Result<String> {
        if !input.is_empty() {
            let input = remove_enter(&input);
            self.stdin.write_all(input.as_bytes()).await?;
        }
        drop(self.stdin);
        let out = self.proc.wait_with_output().await?.stdout;
        if out.is_empty() {
            return Err(EmptyRet);
        }
        let out = String::from_utf8(out)?;
        if matches!(out.as_bytes(), [ERROR_SIG_CHAR, ..]) {
            error!(run_err=%out, %input, "child worker exec failed");
            return Err(RunSubCmdError(out));
        }
        Ok(out)
    }
}

impl Worker {
    fn init(cmd: &Arc<Cmd>) -> Result<Self> {
        let cmd = cmd.clone();
        let mut child_def = Command::new(&cmd.bin);
        if let Some(args) = &cmd.args {
            child_def.args(args);
        }
        let mut proc = child_def
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // .stderr(Stdio::inherit())
            .spawn()?;
        let stdin = proc.stdin.take().expect("Failed to open stdin");
        let stdout = BufReader::with_capacity(
            BUFF_SIZE,
            proc.stdout.take().expect("Failed to open stdout"),
        );
        Ok(Self {
            cmd,
            proc,
            stdin,
            stdout,
            ts: Instant::now(),
        })
    }

    async fn process(&mut self, input: String) -> Result<String> {
        if let Ok(Some(_)) | Err(_) = self.proc.try_wait() {
            // proc has exited
            info!("child worker has exited");
            *self = Worker::init(&self.cmd)?;
        } else {
            self.ts = Instant::now();
        }
        let input = remove_enter(&input);
        self.stdin.write_all(input.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        let mut ret_buf = String::new();
        let nb = self.stdout.read_line(&mut ret_buf).await?;
        if nb == 0 {
            return Err(EmptyRet);
        }
        trim_end_inplace(&mut ret_buf);
        if ret_buf.is_empty() {
            return Err(EmptyRet);
        }
        if matches!(ret_buf.as_bytes(), [ERROR_SIG_CHAR, ..]) {
            error!(run_err=%ret_buf, %input, "child worker exec failed");
            return Err(RunSubCmdError(ret_buf));
        }
        Ok(ret_buf)
    }
}

impl ChildProc {
    pub async fn setup(
        bin: &str,
        args: Option<Vec<String>>,
        size: usize,
        inter: Option<Duration>,
    ) -> Result<ChildWorker> {
        let cmd = Arc::new(Cmd {
            bin: bin.to_owned(),
            args,
        });
        let mut workers = Vec::new();
        for _ in 0..size {
            workers.push(Worker::init(&cmd)?);
            if let Some(d) = inter {
                tokio::time::sleep(d).await;
            }
        }
        let myself = Arc::new(Self {
            cmd,
            workers: Mutex::new(workers),
            workers_sem: Semaphore::new(size),
            concurrent: size,
        });
        Ok(myself)
    }

    pub async fn one_shot(&self, input: String) -> Result<String> {
        OnceWorker::init(&self.cmd)?.run(input).await
    }

    pub async fn submit(&self, input: String) -> Result<String> {
        if self.concurrent == 0 {
            return self.one_shot(input).await;
        }

        tokio::select! {
            _permit = self.workers_sem.acquire() => {
                let mut worker = self.workers.lock()?.pop().expect("workers not sufficient");
                let ret = worker.process(input).await;
                self.workers.lock()?.push(worker);
                ret
            }
            _ = tokio::time::sleep(timeout()) => Err(SubmitTimeout)
        }
    }
}

impl ChildProcQueue {
    pub async fn setup(
        bin: &str,
        args: Option<Vec<String>>,
        size: usize,
        inter: Option<Duration>,
    ) -> Result<ChildWorkerQueue> {
        let cmd = Arc::new(Cmd {
            bin: bin.to_owned(),
            args,
        });
        let (tx, rx) = flume::bounded(size);
        for _ in 0..size {
            tx.send(Arc::new(Lock::new(Worker::init(&cmd)?)))?;
            if let Some(d) = inter {
                tokio::time::sleep(d).await;
            }
        }
        let myself = Arc::new(Self {
            cmd,
            workers: rx,
            recycler: tx,
            worker_map: Mutex::default(),
            worker_cond: Mutex::default(),
        });
        let bg_one = myself.clone();
        tokio::spawn(bg_one.check_timeout_workers()); // check and clean timeout bound workers in background
        Ok(myself)
    }

    pub async fn bind(&self, sid: String, input: String) -> Result<String> {
        // let mut exist = true;
        let cond = self
            .worker_cond
            .lock()?
            .entry(sid.clone())
            .or_insert_with(|| {
                // exist = false;
                // flume::bounded(1)
                Arc::default()
            })
            .clone();
        // if exist {
        //     cond.1.recv_async().await?;
        // }
        let _guard = cond.lock().await;

        let some_w = self.worker_map.lock()?.get(&sid).cloned();

        if let Some(worker) = some_w {
            // cond.0.send_async(()).await?;
            return worker.lock().await.process(input).await;
        }

        tokio::select! {
            Ok(worker) = self.workers.recv_async() => {
                let ret = worker.lock().await.process(input).await;
                self.worker_map.lock()?.insert(sid, worker);
                ret
            }
            _ = tokio::time::sleep(timeout()) => Err(SubmitTimeout)
        }
        // cond.0.send_async(()).await?;
    }

    pub async fn unbind(&self, sid: String) -> Result<()> {
        let some_w = self.worker_map.lock()?.remove(&sid);
        if let Some(worker) = some_w {
            self.recycler.send_async(worker).await?;
        }
        Ok(())
    }

    async fn check_timeout_workers(self: Arc<Self>) -> Result<()> {
        let mut ticker = tokio::time::interval(BOUND_WORKER_CHECK_INTER);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            let now = ticker.tick().await;
            let bound_workers = self.worker_map.lock()?.clone();
            for (sid, worker) in bound_workers {
                if now - worker.lock().await.ts >= BOUND_WORKER_TIMEOUT {
                    self.unbind(sid).await.ok();
                }
            }
        }
    }
}

fn timeout() -> Duration {
    if let Ok(c) = config::peek_config() {
        if c.py.timeout_secs > 0 {
            return Duration::from_secs(c.py.timeout_secs);
        }
    }
    CMD_SUBMIT_TIMEOUT
}

#[cfg(test)]
mod tests {

    #[tokio::test(flavor = "current_thread")]
    async fn test_one_shot() {
        let cp = super::ChildProc::setup("rev", None, 0, None)
            .await
            .expect("Failed to setup child proc");
        let out = cp
            .one_shot("Hello, world..".to_owned())
            .await
            .expect("Failed to run task");
        assert_eq!(out, "..dlrow ,olleH");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_submit() {
        let cp =
            super::ChildProc::setup("sed", Some(vec!["-u".to_owned(), "".to_owned()]), 2, None)
                .await
                .expect("Failed to setup child workers");
        let out = cp
            .submit("Hello, world..".to_owned())
            .await
            .expect("Failed to run task");
        assert_eq!(out, "Hello, world..");
    }
}
