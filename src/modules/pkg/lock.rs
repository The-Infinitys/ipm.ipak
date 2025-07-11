//! このモジュールは、パッケージ操作のロックとタスク管理を担当します。
//!
//! `ipak`が複数のプロセスで同時にパッケージを操作しようとした際の競合を防ぎ、
//! 安全なパッケージ管理を実現します。

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;
use std::time::{Duration, SystemTime};

use crate::modules::system::path::{global, local};

const LOCK_TIMEOUT: Duration = Duration::from_secs(60); // 1 minute

/// ロックファイルとタスクファイルを管理する構造体
pub struct LockManager {
    lock_path: PathBuf,
    tasks_path: PathBuf,
}

impl LockManager {
    /// 新しい`LockManager`インスタンスを作成します。
    ///
    /// # Arguments
    ///
    /// * `is_global` - グローバルなロックファイルを管理するかどうか
    pub fn new(is_global: bool) -> Self {
        if is_global {
            Self {
                lock_path: global::lock_filepath(),
                tasks_path: global::tasks_filepath(),
            }
        } else {
            Self {
                lock_path: local::lock_filepath(),
                tasks_path: local::tasks_filepath(),
            }
        }
    }

    /// ロックを取得します。
    ///
    /// ロックが既に取得されている場合は、タイムアウトまで待機します。
    /// タイムアウトした場合は、エラーを返します。
    ///
    /// # Returns
    ///
    /// `Ok(())` - ロックの取得に成功した場合
    /// `Err(io::Error)` - ロックの取得に失敗した場合
    pub fn acquire_lock(&self) -> io::Result<()> {
        let start_time = SystemTime::now();
        loop {
            if self.is_lock_stale()? {
                self.clear_stale_lock()?;
                self.run_pending_tasks()?;
            }

            if let Ok(mut file) = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&self.lock_path)
            {
                file.write_all(process::id().to_string().as_bytes())?;
                return Ok(());
            }

            if start_time.elapsed().unwrap_or_default() > LOCK_TIMEOUT {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Failed to acquire lock",
                ));
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// ロックを解放します。
    ///
    /// # Returns
    ///
    /// `Ok(())` - ロックの解放に成功した場合
    /// `Err(io::Error)` - ロックの解放に失敗した場合
    pub fn release_lock(&self) -> io::Result<()> {
        fs::remove_file(&self.lock_path)
    }

    /// タスクを追加します。
    ///
    /// # Arguments
    ///
    /// * `task` - 追加するタスク
    ///
    /// # Returns
    ///
    /// `Ok(())` - タスクの追加に成功した場合
    /// `Err(io::Error)` - タスクの追加に失敗した場合
    pub fn add_task(&self, task: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.tasks_path)?;
        writeln!(file, "{}", task)?;
        Ok(())
    }

    /// 保留中のタスクを実行します。
    fn run_pending_tasks(&self) -> io::Result<()> {
        if !self.tasks_path.exists() {
            return Ok(());
        }

        let mut tasks = String::new();
        File::open(&self.tasks_path)?.read_to_string(&mut tasks)?;

        // TODO: Implement task execution logic here

        fs::remove_file(&self.tasks_path)?;
        Ok(())
    }

    /// ロックが古いかどうかを確認します。
    fn is_lock_stale(&self) -> io::Result<bool> {
        if !self.lock_path.exists() {
            return Ok(false);
        }

        let metadata = fs::metadata(&self.lock_path)?;
        let modified_time = metadata.modified()?;

        Ok(modified_time.elapsed().unwrap_or_default() > LOCK_TIMEOUT)
    }

    /// 古いロックをクリアします。
    fn clear_stale_lock(&self) -> io::Result<()> {
        fs::remove_file(&self.lock_path)
    }
}
