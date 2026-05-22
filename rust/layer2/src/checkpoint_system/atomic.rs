//! # Atomic File Writer
//!
//! 原子文件写入实现，确保数据持久化的完整性。

use std::path::{Path, PathBuf};

use crate::types::{Layer2Error, Layer2Result};

/// 临时文件前缀
const TEMP_FILE_PREFIX: &str = ".tmp_checkpoint_";

/// 原子文件写入器
///
/// 使用临时文件 + 重命名模式实现原子写入。
/// 在 Windows 上使用 os.replace()，在 NTFS 上是原子的。
/// 在 Unix 上使用 rename()，在相同文件系统上是原子的。
pub struct AtomicFileWriter {
    sync_on_write: bool,
    verify_write: bool,
}

impl AtomicFileWriter {
    /// 创建新的原子写入器
    pub fn new() -> Self {
        Self {
            sync_on_write: true,
            verify_write: true,
        }
    }

    /// 配置是否在写入后同步
    pub fn with_sync(mut self, sync: bool) -> Self {
        self.sync_on_write = sync;
        self
    }

    /// 配置是否验证写入
    pub fn with_verify(mut self, verify: bool) -> Self {
        self.verify_write = verify;
        self
    }

    /// 原子写入内容到文件
    ///
    /// # Arguments
    /// * `filepath` - 目标文件路径
    /// * `content` - 要写入的内容
    ///
    /// # Returns
    /// 成功返回 Ok(())，失败返回错误信息
    pub fn write_atomic(&self, filepath: &Path, content: &str) -> Layer2Result<()> {
        let filepath = PathBuf::from(filepath);
        let parent_dir = filepath.parent().ok_or_else(|| {
            Layer2Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid file path",
            ))
        })?;

        // 确保父目录存在
        std::fs::create_dir_all(parent_dir)?;

        // 生成临时文件路径（在同一目录下，保证同一文件系统）
        let temp_filename = format!("{}{}", TEMP_FILE_PREFIX, uuid::Uuid::new_v4());
        let temp_path = parent_dir.join(temp_filename);

        // 执行写入
        let result = self.do_write(&filepath, &temp_path, content);

        // 清理临时文件（如果失败）
        if result.is_err() {
            let _ = std::fs::remove_file(&temp_path);
        }

        result
    }

    fn do_write(&self, filepath: &Path, temp_path: &Path, content: &str) -> Layer2Result<()> {
        use std::fs::File;
        use std::io::Write;

        // 1. 写入临时文件
        {
            let mut file = File::create(temp_path)?;
            file.write_all(content.as_bytes())?;

            // 同步到磁盘
            if self.sync_on_write {
                file.sync_all()?;
            }
        }

        // 2. 验证写入（可选）
        if self.verify_write {
            let written = std::fs::read_to_string(temp_path)?;
            if written != content {
                return Err(Layer2Error::CheckpointCorrupted(
                    "Write verification failed: content mismatch".to_string(),
                )
                .into());
            }
        }

        // 3. 原子重命名
        std::fs::rename(temp_path, filepath)?;

        // 4. 同步目录（Unix only）
        #[cfg(unix)]
        if self.sync_on_write {
            use std::os::unix::fs::OpenOptionsExt;
            let dir_fd = std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_DIRECTORY)
                .open(filepath.parent().unwrap())?;
            dir_fd.sync_all()?;
        }

        Ok(())
    }

    /// 安全删除文件
    pub fn safe_remove(&self, path: &Path) -> Layer2Result<()> {
        match std::fs::remove_file(path) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(Layer2Error::Io(e).into()),
        }
    }
}

impl Default for AtomicFileWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        let writer = AtomicFileWriter::new();
        writer.write_atomic(&file_path, "test content").unwrap();

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_atomic_write_creates_parent() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nested/dir/test.json");

        let writer = AtomicFileWriter::new();
        writer.write_atomic(&file_path, "test").unwrap();

        assert!(file_path.exists());
    }
}
