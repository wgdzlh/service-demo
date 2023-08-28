use std::collections::HashMap;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as Lock;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Semaphore;
use tokio::time::{Duration, Instant};

use crate::app::utils::{remove_enter, trim_end_inplace};
use crate::infrastructure::config;
use crate::repository::Error::{EmptyRet, RunSubCmdError, SubmitTimeout};
use crate::{app::log::*, repository::Result};

const BUFF_SIZE: usize = 65536;
const CMD_SUBMIT_TIMEOUT: Duration = Duration::from_secs(5);
const BOUND_WORKER_TIMEOUT: Duration = Duration::from_secs(660);
const BOUND_WORKER_CHECK_INTER: Duration = Duration::from_secs(60);

const ERROR_SIG_CHAR: u8 = b'!'; // special first char to indicate child worker has output error info

pub struct Cmd {
    bin: String,
    args: Option<Vec<String>>,
}

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
    worker_cond: Mutex<HashMap<String, CondPair>>,
}

type SharedWorker = Arc<Lock<Worker>>;
type CondPair = (flume::Sender<()>, flume::Receiver<()>);

struct Worker {
    cmd: Arc<Cmd>,
    proc: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    ts: Instant,
}

impl Worker {
    fn start(cmd: &Arc<Cmd>) -> Result<Self> {
        let cmd = cmd.clone();
        let mut child_def = Command::new(&cmd.bin);
        if let Some(args) = &cmd.args {
            child_def.args(args);
        }
        let mut proc = child_def
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
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

    async fn run(mut self, input: String) -> Result<String> {
        if !input.is_empty() {
            let input = remove_enter(&input);
            self.stdin.write_all(input.as_bytes()).await?;
            self.stdin.write_all(b"\n").await?;
        }
        drop(self.stdin);
        let out = self.proc.wait_with_output().await?.stdout;
        if out.is_empty() {
            return Err(EmptyRet);
        }
        let out = String::from_utf8_lossy(&out).into_owned();
        if matches!(out.as_bytes(), [ERROR_SIG_CHAR, ..]) {
            error!(run_err=%out, %input, "child worker exec failed");
            return Err(RunSubCmdError(out));
        }
        Ok(out)
    }

    async fn process(&mut self, input: String) -> Result<String> {
        if let Ok(Some(_)) | Err(_) = self.proc.try_wait() {
            // proc has exited
            info!("child worker has exited");
            *self = Worker::start(&self.cmd)?;
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
    ) -> Result<ChildProc> {
        let cmd = Arc::new(Cmd {
            bin: bin.to_owned(),
            args,
        });
        let mut workers = Vec::new();
        for _ in 0..size {
            workers.push(Worker::start(&cmd)?);
            if let Some(d) = inter {
                tokio::time::sleep(d).await;
            }
        }
        Ok(Self {
            cmd,
            workers: Mutex::new(workers),
            workers_sem: Semaphore::new(size),
            concurrent: size,
        })
    }

    pub async fn one_shot(&self, input: String) -> Result<String> {
        Worker::start(&self.cmd)?.run(input).await
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
    ) -> Result<ChildProcQueue> {
        let cmd = Arc::new(Cmd {
            bin: bin.to_owned(),
            args,
        });
        let (tx, rx) = flume::bounded(size);
        for _ in 0..size {
            tx.send(Arc::new(Lock::new(Worker::start(&cmd)?)))?;
            if let Some(d) = inter {
                tokio::time::sleep(d).await;
            }
        }
        Ok(Self {
            cmd,
            workers: rx,
            recycler: tx,
            worker_map: Default::default(),
            worker_cond: Default::default(),
        })
    }

    pub async fn bind(&self, sid: String, input: String) -> Result<String> {
        let mut exist = true;
        let pair = self
            .worker_cond
            .lock()?
            .entry(sid.clone())
            .or_insert_with(|| {
                exist = false;
                flume::bounded(1)
            })
            .clone();
        if exist {
            pair.1.recv_async().await?;
        }

        let ret;
        let some_w = self.worker_map.lock()?.get(&sid).cloned();

        if let Some(worker) = some_w {
            let ret = worker.lock().await.process(input).await;
            pair.0.send_async(()).await?;
            return ret;
        }

        tokio::select! {
            Ok(worker) = self.workers.recv_async() => {
                ret = worker.lock().await.process(input).await;
                self.worker_map.lock()?.insert(sid, worker);
            }
            _ = tokio::time::sleep(timeout()) => {
                ret = Err(SubmitTimeout);
            }
        }
        pair.0.send_async(()).await?;
        ret
    }

    pub async fn unbind(&self, sid: String) -> Result<()> {
        let some_w = self.worker_map.lock()?.remove(&sid);
        if let Some(worker) = some_w {
            self.recycler.send_async(worker).await?;
        }
        Ok(())
    }

    pub async fn check_timeout_workers(&self) -> Result<()> {
        let mut ticker = tokio::time::interval(BOUND_WORKER_CHECK_INTER);
        loop {
            let now = ticker.tick().await;
            let bound_workers = self.worker_map.lock()?.clone();
            for (sid, worker) in bound_workers {
                if now - worker.lock().await.ts >= BOUND_WORKER_TIMEOUT {
                    self.unbind(sid).await?;
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
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::process::Command;

    #[tokio::test(flavor = "current_thread")]
    async fn test() {
        let mut child = Command::new("rev")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn child process");

        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        // tokio::spawn(async move {
        stdin
            .write_all("Hello, world!".as_bytes())
            .await
            .expect("Failed to write to stdin");
        // });
        drop(stdin);

        let stdout = child.stdout.take().expect("Failed to open stdout");
        // let output = child
        //     .wait_with_output()
        //     .await
        //     .expect("Failed to read stdout");
        let mut output = BufReader::new(stdout);
        let mut ret_buf = String::new();
        output
            .read_line(&mut ret_buf)
            .await
            .expect("Failed to read stdout");
        // assert_eq!(String::from_utf8_lossy(&output.stdout), "!dlrow ,olleH");
        assert_eq!(ret_buf, "!dlrow ,olleH");
    }
}
