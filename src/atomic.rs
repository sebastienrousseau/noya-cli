// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! Atomic file replacement shared by `noyafmt --write` and
//! `noyavalidate --fix`.

use std::fs::{File, Permissions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Write `bytes` to `path` without a window in which the file is
/// truncated or half-written.
///
/// The bytes go to a temporary file in the same directory, are synced,
/// take over the target's permissions when it exists, and are renamed
/// over it; rename is atomic on every platform the binaries ship for.
/// An interrupted `noyafmt --write` or `noyavalidate --fix` therefore
/// leaves either the old file or the new one, never a torn file, and
/// the temporary is removed on any failure.
///
/// # Errors
///
/// Any I/O error from creating, writing, syncing or renaming the
/// temporary file, in which case `path` is untouched.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    };
    let stem = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("noya-cli");
    let perms = std::fs::metadata(path).ok().map(|m| m.permissions());
    let (tmp, mut f) = create_temp(&parent, stem, perms.as_ref())?;
    let result = (|| {
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// How many fresh names to try before giving up on a directory where
/// every candidate already exists.
const TEMP_ATTEMPTS: u32 = 16;

/// Create a new temporary file next to the target under an
/// unpredictable name, retrying on a name collision.
fn create_temp(dir: &Path, stem: &str, perms: Option<&Permissions>) -> io::Result<(PathBuf, File)> {
    let mut last = None;
    for _ in 0..TEMP_ATTEMPTS {
        let tmp = dir.join(format!(".{stem}.{}.tmp", random_suffix()));
        match open_temp(&tmp, perms) {
            Ok(f) => return Ok((tmp, f)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => last = Some(e),
            Err(e) => return Err(e),
        }
    }
    Err(last.unwrap_or_else(|| io::Error::other("no temporary file name available")))
}

/// 128 bits from the standard library's randomly keyed hasher, so the
/// temporary name cannot be guessed ahead of time.
fn random_suffix() -> String {
    use std::hash::{BuildHasher, Hasher};
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let mut out = 0u128;
    for round in 0..2u8 {
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u128(nanos);
        h.write_u8(round);
        out = (out << 64) | u128::from(h.finish());
    }
    format!("{out:032x}")
}

/// Open the temporary file at `tmp`, ready for the new contents.
///
/// The open is exclusive (`O_EXCL`), so it fails instead of following
/// a symlink or reusing a file someone planted at that name. On unix
/// the file is created with the target's mode (narrowed further by the
/// umask) and then set to exactly that mode, all before a byte is
/// written, so the contents are never readable by anyone the target
/// excludes.
fn open_temp(tmp: &Path, perms: Option<&Permissions>) -> io::Result<File> {
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        if let Some(p) = perms {
            opts.mode(p.mode() & 0o777);
        }
    }
    let f = opts.open(tmp)?;
    if let Some(p) = perms {
        if let Err(e) = f.set_permissions(p.clone()) {
            drop(f);
            let _ = std::fs::remove_file(tmp);
            return Err(e);
        }
    }
    Ok(f)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("noya_cli_atomic_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn temp_is_never_more_permissive_than_the_target() {
        let dir = scratch("mode");
        let tmp = dir.join(".t.tmp");
        let perms = Permissions::from_mode(0o600);
        let f = open_temp(&tmp, Some(&perms)).unwrap();
        // Checked before a single byte is written.
        let mode = f.metadata().unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "temp opened at {mode:o} before the write");
    }

    #[test]
    fn a_planted_symlink_at_the_temp_name_is_not_followed() {
        let dir = scratch("symlink");
        let victim = dir.join("victim.txt");
        std::fs::write(&victim, "original").unwrap();
        let tmp = dir.join(".t.tmp");
        std::os::unix::fs::symlink(&victim, &tmp).unwrap();
        let res = open_temp(&tmp, None).and_then(|mut f| f.write_all(b"planted"));
        assert!(res.is_err(), "open followed the planted symlink");
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "original");
    }

    #[test]
    fn create_temp_names_are_unique_and_unpredictable() {
        let dir = scratch("names");
        let (a, _fa) = create_temp(&dir, "x.yaml", None).unwrap();
        let (b, _fb) = create_temp(&dir, "x.yaml", None).unwrap();
        assert_ne!(a, b);
        let name = a.file_name().unwrap().to_str().unwrap();
        // ".x.yaml." + 32 hex digits + ".tmp"
        assert_eq!(name.len(), ".x.yaml.".len() + 32 + ".tmp".len(), "{name}");
    }

    #[test]
    fn write_atomic_keeps_owner_only_and_executable_modes() {
        let dir = scratch("keep");
        for mode in [0o600, 0o755, 0o640] {
            let target = dir.join(format!("f{mode:o}.yaml"));
            std::fs::write(&target, "a: 1\n").unwrap();
            std::fs::set_permissions(&target, Permissions::from_mode(mode)).unwrap();
            write_atomic(&target, b"a: 2\n").unwrap();
            let got = std::fs::metadata(&target).unwrap().permissions().mode() & 0o777;
            assert_eq!(got, mode, "mode {mode:o} became {got:o}");
            assert_eq!(std::fs::read_to_string(&target).unwrap(), "a: 2\n");
        }
        // Only the targets remain: no temporary is left behind.
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 3);
    }

    #[test]
    fn write_atomic_into_a_missing_directory_fails_cleanly() {
        let dir = scratch("missing");
        let target = dir.join("no-such-dir").join("f.yaml");
        assert!(write_atomic(&target, b"a: 1\n").is_err());
    }
}
