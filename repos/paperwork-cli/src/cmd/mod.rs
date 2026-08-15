//! Command modules and shared context.

pub mod brief;
pub mod contacts;
pub mod post;
pub mod profile;
pub mod validate;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use crate::output::OutputMode;

/// Shared context for all commands (stateless -- no workspace root).
pub struct Context {
    /// Output mode.
    pub mode: OutputMode,
    /// Suppress confirmation messages.
    pub quiet: bool,
}

/// Resolve the operation target path via three-stage parsing (v0.5 semantics):
///
/// 1. The given path exists as-is and is a **file** (`is_file()`; directories
///    never match) -> use the original path unchanged.
/// 2. Otherwise, the type-suffixed variant exists as a file -> use it.
/// 3. Neither exists -> return the suffixed variant as the operation landing
///    path.
///
/// This only decides the path. Physical file creation happens exclusively in
/// write commands (send/create/add); read-only commands (read/summary/validate)
/// report not-found when all three stages miss.
pub fn ensure_suffix(path: PathBuf, suffix: &str) -> PathBuf {
    // Stage 1: original path exists as a file -> use as-is.
    //
    // B-5 platform dependency (review S-2 note): on Windows, `is_file()`
    // returns false for a bare reserved device name such as `CON` or `NUL`
    // (it is a device, not a file), so stage 1 misses and the name falls
    // through to suffix normalization (`CON` -> `CON.post.md`), sealing the
    // device surface. The behaviour anchor is the cli_integration test
    // `reserved_device_names_are_sealed_by_suffix_normalization`; do not
    // "simplify" stage 1 without re-checking that anchor.
    if path.is_file() {
        return path;
    }
    // Stage 2/3: suffixed variant is the fallback (existing file wins).
    let suffixed = suffixed_variant(&path, suffix);
    if suffixed.is_file() {
        return suffixed;
    }
    suffixed
}

/// Compute the type-suffixed variant of a path:
/// already ends with the suffix -> unchanged; bare `.md` -> replaced;
/// otherwise -> appended.
///
/// NEW-3 (P-6): every step stays in the native `OsStr` representation — no
/// `to_string_lossy()` roundtrip, which replaced invalid Unicode sequences
/// with U+FFFD and could silently redirect the write to a wrong file name.
fn suffixed_variant(path: &Path, suffix: &str) -> PathBuf {
    let suffix_os = OsStr::new(suffix);
    if os_ends_with(path.as_os_str(), suffix_os) {
        return path.to_path_buf();
    }
    if let Some(base) = os_strip_suffix(path.as_os_str(), OsStr::new(".md")) {
        let mut out = base;
        out.push(suffix);
        return PathBuf::from(out);
    }
    let mut out = path.as_os_str().to_os_string();
    out.push(suffix);
    PathBuf::from(out)
}

/// Does `os` end with `suffix`? Compared on the platform-native encoded
/// bytes so non-UTF-8 components participate exactly as stored (NEW-3).
pub(crate) fn os_ends_with(os: &OsStr, suffix: &OsStr) -> bool {
    os.as_encoded_bytes().ends_with(suffix.as_encoded_bytes())
}

/// Strip a trailing `suffix` from `os`, returning the prefix verbatim.
/// Reconstruction is platform-native (raw bytes on Unix, UTF-16 units on
/// Windows), so components that are not valid Unicode survive untouched
/// (NEW-3).
#[cfg(not(windows))]
pub(crate) fn os_strip_suffix(os: &OsStr, suffix: &OsStr) -> Option<OsString> {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let bytes = os.as_bytes();
    let tail = suffix.as_bytes();
    bytes
        .ends_with(tail)
        .then(|| OsStringExt::from_vec(bytes[..bytes.len() - tail.len()].to_vec()))
}

#[cfg(windows)]
pub(crate) fn os_strip_suffix(os: &OsStr, suffix: &OsStr) -> Option<OsString> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    let wide: Vec<u16> = os.encode_wide().collect();
    let tail: Vec<u16> = suffix.encode_wide().collect();
    wide.ends_with(&tail)
        .then(|| OsStringExt::from_wide(&wide[..wide.len() - tail.len()]))
}

#[cfg(test)]
mod tests {
    use super::ensure_suffix;
    use std::path::PathBuf;

    #[test]
    fn ensure_suffix_appends_when_missing() {
        assert_eq!(
            ensure_suffix(PathBuf::from("alice"), ".profile.md"),
            PathBuf::from("alice.profile.md")
        );
    }

    #[test]
    fn ensure_suffix_replaces_bare_md() {
        assert_eq!(
            ensure_suffix(PathBuf::from("alice.md"), ".profile.md"),
            PathBuf::from("alice.profile.md")
        );
    }

    #[test]
    fn ensure_suffix_keeps_already_suffixed() {
        assert_eq!(
            ensure_suffix(PathBuf::from("alice.profile.md"), ".profile.md"),
            PathBuf::from("alice.profile.md")
        );
    }

    /// NEW-3 regression lock: paths containing non-Unicode components must
    /// roundtrip through suffix handling byte-for-byte — the old
    /// `to_string_lossy()` roundtrip rewrote them with U+FFFD. (Ported from
    /// the wip snapshot; the three-stage existence semantics stay master's.)
    #[cfg(not(windows))]
    mod invalid_utf8 {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        use std::path::PathBuf;

        use super::super::ensure_suffix;

        fn raw(bytes: &[u8]) -> OsString {
            OsStringExt::from_vec(bytes.to_vec())
        }

        #[test]
        fn append_preserves_invalid_bytes() {
            let path = PathBuf::from(raw(b"al\xFFice"));
            let mut expected = raw(b"al\xFFice");
            expected.push(".post.md");
            assert_eq!(
                ensure_suffix(path, ".post.md").as_os_str(),
                expected.as_os_str()
            );
        }

        #[test]
        fn md_replacement_preserves_invalid_bytes() {
            let path = PathBuf::from(raw(b"al\xFFice.md"));
            let mut expected = raw(b"al\xFFice");
            expected.push(".post.md");
            assert_eq!(
                ensure_suffix(path, ".post.md").as_os_str(),
                expected.as_os_str()
            );
        }

        #[test]
        fn already_suffixed_preserves_invalid_bytes() {
            let path = PathBuf::from(raw(b"al\xFFice.post.md"));
            let expected = path.clone();
            assert_eq!(ensure_suffix(path, ".post.md"), expected);
        }
    }

    /// Windows OsStr is UTF-16; an unpaired surrogate is a legal path
    /// component but not representable as `str` — exactly the NEW-3 shape.
    #[cfg(windows)]
    mod invalid_utf8 {
        use std::ffi::OsString;
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        use std::path::PathBuf;

        use super::super::ensure_suffix;

        /// `al<unpaired surrogate>ice` — not convertible to UTF-8.
        const STEM: &[u16] = &[
            b'a' as u16,
            b'l' as u16,
            0xD800,
            b'i' as u16,
            b'c' as u16,
            b'e' as u16,
        ];

        fn raw(wide: &[u16]) -> OsString {
            OsStringExt::from_wide(wide)
        }

        #[test]
        fn append_preserves_unpaired_surrogate() {
            let path = PathBuf::from(raw(STEM));
            let mut expected = raw(STEM);
            expected.push(".post.md");
            let out = ensure_suffix(path, ".post.md");
            assert_eq!(out.as_os_str(), expected.as_os_str());
            let wide: Vec<u16> = out.file_name().unwrap().encode_wide().collect();
            assert!(
                wide.contains(&0xD800),
                "surrogate must survive the roundtrip"
            );
        }

        #[test]
        fn md_replacement_preserves_unpaired_surrogate() {
            let mut stem_md: Vec<u16> = STEM.to_vec();
            stem_md.extend(".md".encode_utf16());
            let path = PathBuf::from(raw(&stem_md));
            let mut expected = raw(STEM);
            expected.push(".post.md");
            let out = ensure_suffix(path, ".post.md");
            assert_eq!(out.as_os_str(), expected.as_os_str());
        }

        #[test]
        fn already_suffixed_preserves_unpaired_surrogate() {
            let mut stem_md: Vec<u16> = STEM.to_vec();
            stem_md.extend(".post.md".encode_utf16());
            let path = PathBuf::from(raw(&stem_md));
            let expected = path.clone();
            assert_eq!(ensure_suffix(path, ".post.md"), expected);
        }
    }
}
