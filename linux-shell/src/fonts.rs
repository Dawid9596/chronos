use std::ffi::CString;
use std::io::Read;
use std::os::raw::c_void;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// Bundled fontconfig app-font registration.
// Fontconfig ships with the GTK/Pango stack on both Linux and the MSYS2
// ucrt64 toolchain used for Windows builds, so we link it directly instead
// of adding a crate dependency.
#[link(name = "fontconfig")]
unsafe extern "C" {
    /// FcBool FcConfigAppFontAddFile(FcConfig *config, const FcChar8 *file);
    /// Passing NULL for config uses the current configuration.
    #[link_name = "FcConfigAppFontAddFile"]
    fn fc_config_app_font_add_file(config: *mut c_void, file: *const u8) -> i32;
    #[link_name = "FcInit"]
    fn fc_init() -> i32;
}

const EMOJI_FONT: &[u8] = include_bytes!("../../assets/fonts/NotoColorEmoji.ttf");
const FONT_FILE_NAME: &str = "NotoColorEmoji.ttf";
const HASH_FILE_NAME: &str = "NotoColorEmoji.ttf.sha";

// FNV-1a 64-bit constants. A fast non-cryptographic digest is enough here:
// the hash only has to answer "does the extracted file still equal the bytes
// compiled into this binary?", not resist tampering.
const FNV_OFFSET: u64 = 0xcbf29ce4_84222325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Digest of the bytes compiled into this binary, computed at most once.
fn embedded_font_hash() -> u64 {
    static HASH: OnceLock<u64> = OnceLock::new();
    *HASH.get_or_init(|| fnv1a64(EMOJI_FONT))
}

/// Reads the hex digest from the sidecar file, if it exists and parses.
fn stored_hash(path: &Path) -> Option<u64> {
    let text = std::fs::read_to_string(path).ok()?;
    u64::from_str_radix(text.trim(), 16).ok()
}

/// Hashes an on-disk file in streaming chunks (the font is ~10 MB).
fn hash_file_contents(path: &Path) -> Option<u64> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut hash = FNV_OFFSET;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        for &byte in &buffer[..read] {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    Some(hash)
}

/// True when the extracted font is present, matches the compiled-in font's
/// size, its sidecar records the compiled-in hash, AND its actual content
/// hashes to the same value (so an in-place corruption or a same-length
/// font update is detected as stale rather than trusted).
fn extracted_font_is_current(font_path: &Path, hash_path: &Path) -> bool {
    let expected = embedded_font_hash();
    let size_matches = match std::fs::metadata(font_path) {
        Ok(meta) => meta.is_file() && meta.len() == EMOJI_FONT.len() as u64,
        Err(_) => false,
    };
    size_matches
        && stored_hash(hash_path) == Some(expected)
        && hash_file_contents(font_path) == Some(expected)
}

/// Extracts the bundled emoji font and its hash sidecar into `base_dir`
/// (rewriting any stale or partially written copy). Extraction goes through
/// temp files + rename, so a crash mid-write can never leave a half-truncated
/// font that would be picked up and trusted on the next launch. Returns the
/// font path, or None when the directory could not be prepared.
fn ensure_font_file(base_dir: &Path) -> Option<PathBuf> {
    std::fs::create_dir_all(base_dir).ok()?;

    let font_path = base_dir.join(FONT_FILE_NAME);
    let hash_path = base_dir.join(HASH_FILE_NAME);
    if extracted_font_is_current(&font_path, &hash_path) {
        // Rename order note: the font is replaced before the sidecar, so a
        // crash between the two renames at worst leaves an old sidecar;
        // the content check above then forces a clean rewrite next launch.
        return Some(font_path);
    }

    let expected = embedded_font_hash();
    let font_tmp = base_dir.join(format!("{FONT_FILE_NAME}.tmp"));
    let hash_tmp = base_dir.join(format!("{HASH_FILE_NAME}.tmp"));

    std::fs::write(&font_tmp, EMOJI_FONT).ok()?;
    // std::fs::rename maps to MoveFileEx with MOVEFILE_REPLACE_EXISTING on
    // Windows, so this replaces an existing font file in one step.
    std::fs::rename(&font_tmp, &font_path).ok()?;

    std::fs::write(&hash_tmp, format!("{expected:016x}")).ok()?;
    std::fs::rename(&hash_tmp, &hash_path).ok()?;

    Some(font_path)
}

/// Extracts the bundled emoji font to the user's cache directory (refreshing
/// it whenever the compiled-in copy changes, verified by content hash) and
/// registers it with fontconfig as an application font, so every family name
/// of the font (e.g. "Noto Color Emoji") resolves even when the font is not
/// installed system-wide (e.g. the packaged Windows build).
///
/// Must run before GTK/Pango creates its font map, i.e. at the very start
/// of `main()`.
pub fn register_bundled_fonts() {
    // Candidate locations, in order of preference.
    let cache = dirs::cache_dir().map(|d| d.join("chronos"));
    let temp_fallback = std::env::temp_dir().join("chronos-emoji");
    let candidates = [cache, Some(temp_fallback)];

    for dir in candidates.into_iter().flatten() {
        let Some(font_path) = ensure_font_file(&dir) else {
            continue;
        };

        // The path handed to fontconfig must survive an exact UTF-8
        // round-trip: fontconfig on Windows feeds FcChar8 through
        // MultiByteToWideChar(CP_UTF8), which reproduces the string
        // byte-for-byte only if it is valid UTF-8. to_str() (NOT
        // to_string_lossy()) fails outright when the OS path contains
        // unpaired UTF-16 surrogates, so a mangled, non-existent path is
        // never handed to fontconfig; we fall back to the next location
        // instead.
        let Some(path) = font_path.to_str() else {
            continue;
        };
        let Ok(cpath) = CString::new(path) else {
            continue;
        };

        unsafe {
            if fc_init() == 0 {
                continue;
            }
            if fc_config_app_font_add_file(
                std::ptr::null_mut(),
                cpath.as_ptr() as *const u8,
            ) != 0
            {
                return; // registered successfully
            }
        }
        // Registration failed for another reason (e.g. bad file content);
        // try the next location.
    }

    eprintln!(
        "register_bundled_fonts: emoji font not registered - flag/gear \
         emoji fall back to the system font stack"
    );
}
