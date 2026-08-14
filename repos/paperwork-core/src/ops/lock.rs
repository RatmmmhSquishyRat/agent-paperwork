//! RAII guard for fs2-locked managed files (T3 shared infrastructure).
//!
//! `thread_send` / `thread_edit` (and later T4 call sites) all repeat the
//! same shape: open → `lock_exclusive` → read/rewrite → manual
//! `file.unlock().ok()` on every early return. [`LockedFile`] collapses the
//! shape into a guard: the lock is acquired once at construction and
//! released exactly once by `Drop`, no matter how many early returns the
//! operation takes.
//!
//! Error wording stays caller-owned: every fallible step receives a
//! caller-supplied `io::Error -> PaperworkError` mapper, because the
//! per-site fix/example wording is part of the output contract (the same
//! stance as [`PaperworkError::io_ctx`]).

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use fs2::FileExt;

use crate::error::{PaperworkError, Result};

/// An exclusively locked file handle; `Drop` releases the lock.
pub(crate) struct LockedFile {
    file: File,
}

impl LockedFile {
    /// Take ownership of an already-open `file` and acquire the exclusive
    /// fs2 lock. `lock_ctx` maps a lock failure to the caller's IoContext
    /// wording (e.g. "another process may hold the lock; retry shortly").
    pub(crate) fn acquire(
        file: File,
        lock_ctx: impl FnOnce(std::io::Error) -> PaperworkError,
    ) -> Result<Self> {
        file.lock_exclusive().map_err(lock_ctx)?;
        Ok(Self { file })
    }

    /// Borrow the underlying handle — escape hatch for the byte-level call
    /// sites (tail scan, last-byte probe) whose semantics cannot be
    /// expressed through [`read_to_string`](Self::read_to_string) /
    /// [`rewrite`](Self::rewrite). The lock stays held; the caller must not
    /// unlock.
    pub(crate) fn file(&self) -> &File {
        &self.file
    }

    /// Read the whole file content from offset 0 (the handle's current
    /// position is restored to the start first). `ctx` maps every IO
    /// failure inside the method; call sites needing distinct wording per
    /// step use [`file()`](Self::file) instead.
    pub(crate) fn read_to_string(
        &self,
        ctx: impl Fn(std::io::Error) -> PaperworkError,
    ) -> Result<String> {
        let mut handle = &self.file;
        handle.seek(SeekFrom::Start(0)).map_err(&ctx)?;
        let mut content = String::new();
        handle.read_to_string(&mut content).map_err(&ctx)?;
        Ok(content)
    }

    /// Truncate and rewrite the file in place, under the lock
    /// (`set_len(0)` + `seek(0)` + `write_all` — the exact shape of the
    /// historical `thread_edit` rewrite). `ctx` maps every IO failure.
    ///
    /// T4 exemption: no production site can wire this helper — every RMW
    /// site's three IO steps carry DISTINCT verbatim fix wordings, while
    /// `rewrite` maps all steps through a single `ctx`; the mandated
    /// escape hatch is [ile()](Self::file). Kept (with its baseline
    /// unit test) as the capability reference for the truncate+seek+write
    /// shape; removing it would delete a pinned baseline assertion.
    #[allow(dead_code)] // T4 exemption: single-ctx vs per-step wording
    pub(crate) fn rewrite(
        &self,
        new_content: &str,
        ctx: impl Fn(std::io::Error) -> PaperworkError,
    ) -> Result<()> {
        self.file.set_len(0).map_err(&ctx)?;
        let mut handle = &self.file;
        handle.seek(SeekFrom::Start(0)).map_err(&ctx)?;
        handle.write_all(new_content.as_bytes()).map_err(&ctx)?;
        Ok(())
    }
}

impl Drop for LockedFile {
    fn drop(&mut self) {
        // Ultra Review F9 wording fix: on the success path the historical
        // `unlock().map_err(...)` converged into this Drop swallowing the
        // error; an unlock failure is effectively unreachable while the
        // handle is valid (the operation result is already decided, and
        // the OS releases the lock with the handle regardless).
        let _ = self.file.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::PaperworkError;
    use std::fs;
    use tempfile::tempdir;

    /// Map every IO error with one fixed wording (tests never pin distinct
    /// per-step wording; the capability is exercised through the API shape).
    fn ctx(path: std::path::PathBuf) -> impl Fn(std::io::Error) -> PaperworkError {
        move |e| PaperworkError::io_ctx(path.clone(), e, "test fix", "")
    }

    fn write_file(path: &std::path::Path, content: &str) {
        let mut f = fs::File::create(path).expect("create");
        f.write_all(content.as_bytes()).expect("write");
    }

    // Lock acquisition: the guard holds an exclusive lock while alive
    // (a second handle cannot even try-lock), and Drop releases it.
    #[test]
    fn locked_file_acquires_and_releases_lock() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("t.post.md");
        write_file(&path, "original");

        let file = fs::File::options()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open");
        let guard = LockedFile::acquire(file, ctx(path.clone())).expect("lock");

        // While the guard is alive, no other handle can take the lock.
        let other = fs::File::options()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open second handle");
        assert!(
            other.try_lock_exclusive().is_err(),
            "second handle must not acquire the lock while LockedFile holds it"
        );

        drop(guard);

        // After Drop the lock is free again.
        let again = fs::File::options()
            .read(true)
            .write(true)
            .open(&path)
            .expect("reopen");
        assert!(
            again.try_lock_exclusive().is_ok(),
            "lock must be free after LockedFile::drop"
        );
        again.unlock().expect("unlock");
    }

    // read_to_string starts from offset 0 regardless of handle position.
    #[test]
    fn locked_file_reads_whole_content() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("t.post.md");
        write_file(&path, "# t\n\nbody\n");

        let file = fs::File::options()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open");
        let guard = LockedFile::acquire(file, ctx(path.clone())).expect("lock");
        assert_eq!(
            guard.read_to_string(ctx(path.clone())).expect("read"),
            "# t\n\nbody\n"
        );
        // Repeatable: the cursor is rewound on every call.
        assert_eq!(
            guard.read_to_string(ctx(path.clone())).expect("read again"),
            "# t\n\nbody\n"
        );
    }

    // rewrite replaces the full content (truncate + seek + write).
    #[test]
    fn locked_file_rewrite_replaces_content() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("t.post.md");
        write_file(&path, "OLD CONTENT, much longer than the replacement");

        let file = fs::File::options()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open");
        let guard = LockedFile::acquire(file, ctx(path.clone())).expect("lock");
        guard.rewrite("new\n", ctx(path.clone())).expect("rewrite");

        // Visible through the same handle...
        assert_eq!(
            guard.read_to_string(ctx(path.clone())).expect("read"),
            "new\n"
        );
        drop(guard);
        // ...and on disk once the lock is released (on Windows the fs2
        // lock blocks foreign handles, so the disk probe waits for drop).
        assert_eq!(fs::read_to_string(&path).expect("disk read"), "new\n");
    }

    // Drop unlocks: a fresh acquire on the same path succeeds afterwards.
    #[test]
    fn locked_file_reacquire_after_drop() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("t.post.md");
        write_file(&path, "x");

        let first = fs::File::options()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open");
        let guard = LockedFile::acquire(first, ctx(path.clone())).expect("lock first");
        drop(guard);

        let second = fs::File::options()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open second");
        let guard2 = LockedFile::acquire(second, ctx(path.clone())).expect("lock second");
        assert_eq!(guard2.read_to_string(ctx(path.clone())).expect("read"), "x");
    }

    // file() accessor exposes the locked handle without dropping the lock.
    #[test]
    fn locked_file_accessor_keeps_lock() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("t.post.md");
        write_file(&path, "x");

        let file = fs::File::options()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open");
        let guard = LockedFile::acquire(file, ctx(path.clone())).expect("lock");
        assert!(guard.file().metadata().is_ok());

        let other = fs::File::options()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open second handle");
        assert!(other.try_lock_exclusive().is_err());
    }
}
