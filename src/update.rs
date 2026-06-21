//! `leaf update` — self-update to the latest stable GitHub release.
//!
//! Pipeline (⑤ Design): resolve target → detect managed install → fetch latest
//! release → compare versions → resolve asset via dist-manifest.json → download
//! → verify sha256 → extract → atomic self-replace.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context, Result, anyhow, bail};
use semver::Version;
use serde::Deserialize;

/// Target triple captured at build time (see build.rs).
const TARGET: &str = env!("LEAF_TARGET");

/// Current version from Cargo.toml.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run() -> Result<ExitCode> {
    // Resolve the real on-disk path (follow symlinks) before deciding anything,
    // so a Homebrew install symlinked into /usr/local/bin is still detected [S3].
    let exe = std::env::current_exe().context("locating the running leaf binary")?;
    let exe = std::fs::canonicalize(&exe).unwrap_or(exe);

    // R5 — refuse to clobber a package-manager-managed binary. Only brew is a
    // confident signal: cargo-dist's own installer also lives in ~/.cargo/bin,
    // so a cargo-bin path is NOT a refuse signal [S3].
    if let Some(manager) = managed_install(&exe) {
        println!(
            "leaf {CURRENT_VERSION}  →  this binary is managed by {}",
            manager.name
        );
        bail!(
            "refusing to overwrite a package-manager-managed install.\n       update with:  {}",
            manager.update_hint
        );
    }

    println!("leaf {CURRENT_VERSION}  →  checking for updates...");
    let release = fetch_latest_release()?;
    println!("latest stable: {}", release.version);

    let current = current_version()?;
    let to = match decide(&current, &release.version) {
        Plan::UpToDate => {
            println!("already up to date ({current})");
            return Ok(ExitCode::SUCCESS);
        }
        Plan::Update { to, .. } => to,
    };

    // Resolve the asset for this platform from the release's own manifest.
    let manifest_url = release
        .assets
        .get("dist-manifest.json")
        .ok_or_else(|| anyhow!("release has no dist-manifest.json"))?;
    let manifest = String::from_utf8(http_get_bytes(manifest_url)?)
        .context("dist-manifest.json is not valid UTF-8")?;
    let names = resolve_asset(&manifest, TARGET)?;
    let archive_url = release
        .assets
        .get(&names.archive)
        .ok_or_else(|| anyhow!("release is missing asset {}", names.archive))?;
    let checksum_url = release
        .assets
        .get(&names.checksum)
        .ok_or_else(|| anyhow!("release is missing asset {}", names.checksum))?;

    let archive = http_get_bytes(archive_url)?;
    println!(
        "downloading  {}  ({:.1} MiB)  ✓",
        names.archive,
        archive.len() as f64 / (1024.0 * 1024.0)
    );

    let sha = String::from_utf8(http_get_bytes(checksum_url)?)
        .context("checksum file is not valid UTF-8")?;
    verify_sha256(&archive, &sha).context("update aborted — your current leaf is unchanged")?;
    println!("verifying    sha256  ✓");

    let new_bytes = extract_binary(&archive, &names.archive)?;

    install_over_self(&exe, &new_bytes)?;
    println!("installed    {current} → {to}  at {}", exe.display());
    Ok(ExitCode::SUCCESS)
}

/// Write the new binary beside the current one and atomically swap it in.
///
/// The temp file shares the install dir so the final swap is a same-filesystem
/// rename. `self_replace` owns the OS-specific move (Unix rename / Windows
/// move-aside) [R7]. We do not auto-escalate to sudo: an unwritable dir is
/// reported with guidance [S3].
fn install_over_self(exe: &Path, new_bytes: &[u8]) -> Result<()> {
    let dir = exe.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::Builder::new()
        .prefix(".leaf-update-")
        .tempfile_in(dir)
        .map_err(|e| permission_aware(dir, e))?;
    tmp.write_all(new_bytes).context("writing new binary")?;
    tmp.flush().ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o755))
            .context("setting executable bit")?;
    }

    let tmp_path = tmp.into_temp_path();
    self_replace::self_replace(&tmp_path).map_err(|e| permission_aware(dir, e))?;
    tmp_path.close().ok();
    Ok(())
}

/// Turn a permission-denied io error into actionable guidance (no auto-sudo).
fn permission_aware(dir: &Path, err: std::io::Error) -> anyhow::Error {
    if err.kind() == std::io::ErrorKind::PermissionDenied {
        anyhow!(
            "no write permission to {}; re-run with privileges or reinstall",
            dir.display()
        )
    } else {
        anyhow::Error::new(err).context("installing the new binary")
    }
}

/// A package manager that owns the binary and should do the update instead.
struct Manager {
    name: &'static str,
    update_hint: &'static str,
}

/// Detect a package-manager-managed install we must not overwrite [S3].
fn managed_install(exe: &Path) -> Option<Manager> {
    if brew_managed(exe) {
        Some(Manager {
            name: "Homebrew",
            update_hint: "brew upgrade leaf",
        })
    } else {
        None
    }
}

/// True if `path` lives under a Homebrew prefix (macOS `/opt/homebrew`, any
/// `/Cellar/`, or linuxbrew). cargo-bin is deliberately NOT treated as managed,
/// because cargo-dist's own installer also targets `~/.cargo/bin` [S3].
fn brew_managed(path: &Path) -> bool {
    let p = path.to_string_lossy();
    p.starts_with("/opt/homebrew/") || p.contains("/Cellar/") || p.contains("/.linuxbrew/")
}

// ── T2: release & asset resolution (pure) ───────────────────────────────────

/// A GitHub release as returned by `/releases/latest`, reduced to what we use.
#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    #[serde(default)]
    assets: Vec<GhAsset>,
}

#[derive(Debug, Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

/// A parsed release: its version and a name → download-url map of assets.
#[derive(Debug)]
struct Release {
    version: Version,
    assets: BTreeMap<String, String>,
}

/// cargo-dist's `dist-manifest.json`, reduced to the artifact map.
#[derive(Debug, Deserialize)]
struct Manifest {
    artifacts: BTreeMap<String, Artifact>,
}

#[derive(Debug, Deserialize)]
struct Artifact {
    kind: String,
    #[serde(default)]
    target_triples: Vec<String>,
    #[serde(default)]
    checksum: Option<String>,
}

/// The asset names resolved for one target triple.
#[derive(Debug, PartialEq, Eq)]
struct AssetNames {
    archive: String,
    checksum: String,
}

/// What to do after comparing the installed version with the latest release.
#[derive(Debug, PartialEq, Eq)]
enum Plan {
    UpToDate,
    Update { from: Version, to: Version },
}

/// Strip an optional leading `v` and parse the tag as semver.
fn parse_tag(tag: &str) -> Result<Version> {
    let raw = tag.strip_prefix('v').unwrap_or(tag);
    Version::parse(raw).with_context(|| format!("release tag is not semver: {tag}"))
}

/// Parse a GitHub `/releases/latest` payload into a [`Release`].
fn parse_release(json: &str) -> Result<Release> {
    let gh: GhRelease = serde_json::from_str(json).context("parsing GitHub release JSON")?;
    let version = parse_tag(&gh.tag_name)?;
    let assets = gh
        .assets
        .into_iter()
        .map(|a| (a.name, a.browser_download_url))
        .collect();
    Ok(Release { version, assets })
}

/// Decide whether to update. Never downgrades: an older or equal latest is a
/// no-op (③ Non-goal — version pin / downgrade out of scope).
fn decide(current: &Version, latest: &Version) -> Plan {
    if latest > current {
        Plan::Update {
            from: current.clone(),
            to: latest.clone(),
        }
    } else {
        Plan::UpToDate
    }
}

/// Resolve the archive + checksum asset names for `target` from a manifest.
///
/// Trust source = the release's own `dist-manifest.json`, not a hardcoded name
/// pattern (④ contract, decision #3): find the `executable-zip` artifact whose
/// `target_triples` contains `target`, and follow its `checksum` field.
fn resolve_asset(manifest_json: &str, target: &str) -> Result<AssetNames> {
    let manifest: Manifest =
        serde_json::from_str(manifest_json).context("parsing dist-manifest.json")?;
    let (archive, artifact) = manifest
        .artifacts
        .iter()
        .find(|(_, a)| a.kind == "executable-zip" && a.target_triples.iter().any(|t| t == target))
        .ok_or_else(|| anyhow!("no release binary for this platform ({target})"))?;
    let checksum = artifact
        .checksum
        .clone()
        .ok_or_else(|| anyhow!("manifest artifact {archive} has no checksum"))?;
    Ok(AssetNames {
        archive: archive.clone(),
        checksum,
    })
}

/// Current version, parsed. Separated so tests don't depend on the build env.
fn current_version() -> Result<Version> {
    parse_tag(CURRENT_VERSION)
}

// Note: there is no hardcoded list of supported targets. The release's own
// dist-manifest.json is the single source of truth (decision #3) — an
// unsupported platform falls out of `resolve_asset` with "no release binary for
// this platform", so the supported set never drifts from what is actually
// distributed.

// ── T3: HTTP download + checksum verification ───────────────────────────────

const REPO: &str = "hoetaek/leaf";
/// Cap downloads well above the ~5 MiB binaries, but bounded.
const MAX_DOWNLOAD_BYTES: u64 = 64 * 1024 * 1024;

fn user_agent() -> String {
    format!("leaf/{CURRENT_VERSION}")
}

/// An https-only agent. `https_only` makes any http hop (including a redirect
/// downgrade) an error — the actual MITM boundary for self-update [S6].
fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .https_only(true)
        .user_agent(user_agent())
        .build()
        .into()
}

/// GET a URL and return the response body as bytes (follows https redirects).
fn http_get_bytes(url: &str) -> Result<Vec<u8>> {
    let mut resp = agent()
        .get(url)
        .call()
        .with_context(|| format!("downloading {url}"))?;
    let bytes = resp
        .body_mut()
        .with_config()
        .limit(MAX_DOWNLOAD_BYTES)
        .read_to_vec()
        .with_context(|| format!("reading body from {url}"))?;
    Ok(bytes)
}

/// Fetch the latest stable release from the GitHub API.
fn fetch_latest_release() -> Result<Release> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let body = agent()
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .call()
        .context("couldn't reach GitHub; check your connection and retry")?
        .body_mut()
        .read_to_string()
        .context("reading GitHub release response")?;
    parse_release(&body)
}

/// Verify `data` against the contents of a cargo-dist `.sha256` file.
///
/// The file is `<hex> *<filename>` (two fields), not bare hex [S1] — compare
/// only the leading hex token, case-insensitively.
fn verify_sha256(data: &[u8], sha256_file: &str) -> Result<()> {
    use sha2::{Digest, Sha256};
    let expected = sha256_file
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("empty .sha256 file"))?
        .to_ascii_lowercase();
    let actual = format!("{:x}", Sha256::digest(data));
    if actual == expected {
        Ok(())
    } else {
        bail!("checksum mismatch (expected {expected}, got {actual})")
    }
}

// ── T4: archive extraction ──────────────────────────────────────────────────

/// Names the leaf executable can have inside a release archive.
const BINARY_NAMES: &[&str] = &["leaf", "leaf.exe"];

fn is_binary_entry(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| BINARY_NAMES.contains(&n))
}

/// Extract the leaf executable bytes from a release archive.
///
/// The binary is not at the archive root [S2]: Unix `.tar.xz` nests it under
/// `leaf-<triple>/leaf`, Windows `.zip` keeps `leaf.exe` flat. Both also carry
/// README/LICENSE/CHANGELOG. So we walk entries and match by basename rather
/// than assuming a path.
fn extract_binary(archive: &[u8], asset_name: &str) -> Result<Vec<u8>> {
    if asset_name.ends_with(".zip") {
        extract_from_zip(archive)
    } else if asset_name.ends_with(".tar.xz") {
        extract_from_tar_xz(archive)
    } else {
        bail!("unknown archive format: {asset_name}")
    }
}

fn extract_from_tar_xz(archive: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut decompressed = Vec::new();
    lzma_rs::xz_decompress(&mut std::io::BufReader::new(archive), &mut decompressed)
        .context("xz-decompressing release archive")?;
    let mut tar = tar::Archive::new(&decompressed[..]);
    for entry in tar.entries().context("reading tar entries")? {
        let mut entry = entry.context("reading tar entry")?;
        let path = entry.path().context("tar entry path")?.into_owned();
        if entry.header().entry_type().is_file() && is_binary_entry(&path) {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .context("reading leaf from tar")?;
            return Ok(buf);
        }
    }
    bail!("no leaf executable found in archive")
}

fn extract_from_zip(archive: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut zip =
        zip::ZipArchive::new(std::io::Cursor::new(archive)).context("opening zip archive")?;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).context("reading zip entry")?;
        let is_bin = file.enclosed_name().as_deref().is_some_and(is_binary_entry);
        if file.is_file() && is_bin {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .context("reading leaf.exe from zip")?;
            return Ok(buf);
        }
    }
    bail!("no leaf executable found in archive")
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = include_str!("../tests/fixtures/dist-manifest-v0.9.1.json");

    #[test]
    fn parses_tag_with_and_without_v() {
        assert_eq!(parse_tag("v0.9.0").unwrap(), Version::new(0, 9, 0));
        assert_eq!(parse_tag("0.9.0").unwrap(), Version::new(0, 9, 0));
        assert!(parse_tag("not-a-version").is_err());
    }

    #[test]
    fn decide_updates_only_upward() {
        let v080 = Version::new(0, 8, 0);
        let v090 = Version::new(0, 9, 0);
        assert_eq!(
            decide(&v080, &v090),
            Plan::Update {
                from: v080.clone(),
                to: v090.clone()
            }
        );
        assert_eq!(decide(&v090, &v090), Plan::UpToDate);
        // never downgrade
        assert_eq!(decide(&v090, &v080), Plan::UpToDate);
    }

    #[test]
    fn resolves_asset_for_each_supported_triple() {
        for (triple, ext) in [
            ("aarch64-apple-darwin", "tar.xz"),
            ("x86_64-apple-darwin", "tar.xz"),
            ("x86_64-unknown-linux-gnu", "tar.xz"),
            ("x86_64-pc-windows-msvc", "zip"),
            // Windows ARM64 — distributed since v0.9.1; resolved from the
            // manifest with no hardcoded allowlist to keep in sync.
            ("aarch64-pc-windows-msvc", "zip"),
        ] {
            let got = resolve_asset(MANIFEST, triple).unwrap();
            assert_eq!(got.archive, format!("leaf-{triple}.{ext}"));
            assert_eq!(got.checksum, format!("leaf-{triple}.{ext}.sha256"));
        }
    }

    #[test]
    fn resolve_asset_rejects_unknown_triple() {
        assert!(resolve_asset(MANIFEST, "powerpc64-unknown-linux-gnu").is_err());
    }

    #[test]
    fn verify_sha256_accepts_cargo_dist_format() {
        // `<hex> *<name>` two-field format, not bare hex [S1].
        let data = b"hello";
        // sha256("hello")
        let hex = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        let file = format!("{hex} *leaf-x86_64-unknown-linux-gnu.tar.xz\n");
        assert!(verify_sha256(data, &file).is_ok());
        // uppercase hex still matches
        assert!(verify_sha256(data, &format!("{} *x", hex.to_uppercase())).is_ok());
        // tampered data fails
        assert!(verify_sha256(b"hellp", &file).is_err());
        // empty file fails
        assert!(verify_sha256(data, "").is_err());
    }

    // Network integration: hits the real v0.9.0 release. Run with
    // `cargo test -- --ignored` or in CI.
    #[test]
    #[ignore = "network: downloads from github releases"]
    fn integration_fetch_latest_and_verify_checksum() {
        let rel = fetch_latest_release().expect("fetch latest");
        assert!(rel.version >= Version::new(0, 9, 0));
        assert!(rel.assets.contains_key("dist-manifest.json"));

        // Resolve the macOS arm64 asset from the live manifest, download it and
        // its checksum, and verify they agree.
        let manifest_url = &rel.assets["dist-manifest.json"];
        let manifest = String::from_utf8(http_get_bytes(manifest_url).unwrap()).unwrap();
        let names = resolve_asset(&manifest, "aarch64-apple-darwin").unwrap();
        let archive = http_get_bytes(&rel.assets[&names.archive]).unwrap();
        let sha = String::from_utf8(http_get_bytes(&rel.assets[&names.checksum]).unwrap()).unwrap();
        verify_sha256(&archive, &sha).expect("real asset checksum matches");

        // A tampered copy must fail.
        let mut bad = archive.clone();
        bad[0] ^= 0xff;
        assert!(verify_sha256(&bad, &sha).is_err());
    }

    #[test]
    fn brew_paths_are_managed_cargo_is_not() {
        use std::path::PathBuf;
        for p in [
            "/opt/homebrew/bin/leaf",
            "/usr/local/Cellar/leaf/0.9.0/bin/leaf",
            "/home/linuxbrew/.linuxbrew/bin/leaf",
        ] {
            assert!(brew_managed(&PathBuf::from(p)), "{p} should be managed");
        }
        for p in [
            "/Users/me/.local/bin/leaf",
            "/Users/me/.cargo/bin/leaf", // NOT refused — cargo-dist uses this too [S3]
            "/usr/bin/leaf",
        ] {
            assert!(
                !brew_managed(&PathBuf::from(p)),
                "{p} should not be managed"
            );
        }
    }

    #[test]
    fn extract_binary_rejects_unknown_format() {
        assert!(extract_binary(b"x", "leaf-foo.7z").is_err());
    }

    // Network integration: downloads real v0.9.0 archives and extracts. Proves
    // lzma-rs decodes the actual cargo-dist .tar.xz [S7] and the basename walk
    // finds the binary in both the nested tar and the flat zip [S2].
    #[test]
    #[ignore = "network: downloads from github releases"]
    fn integration_extract_real_archives() {
        let rel = fetch_latest_release().unwrap();
        let manifest =
            String::from_utf8(http_get_bytes(&rel.assets["dist-manifest.json"]).unwrap()).unwrap();

        // Unix nested .tar.xz → lzma-rs + tar
        let tar_names = resolve_asset(&manifest, "x86_64-unknown-linux-gnu").unwrap();
        let tar_archive = http_get_bytes(&rel.assets[&tar_names.archive]).unwrap();
        let bin = extract_binary(&tar_archive, &tar_names.archive).unwrap();
        assert!(
            bin.len() > 1_000_000,
            "extracted linux binary looks too small"
        );
        assert_eq!(&bin[..4], b"\x7fELF", "extracted file is not an ELF binary");

        // Windows flat .zip
        let zip_names = resolve_asset(&manifest, "x86_64-pc-windows-msvc").unwrap();
        let zip_archive = http_get_bytes(&rel.assets[&zip_names.archive]).unwrap();
        let exe = extract_binary(&zip_archive, &zip_names.archive).unwrap();
        assert!(exe.len() > 1_000_000, "extracted leaf.exe looks too small");
        assert_eq!(&exe[..2], b"MZ", "extracted file is not a PE binary");
    }

    #[test]
    fn parse_release_reads_tag_and_assets() {
        let json = r#"{
            "tag_name": "v0.9.0",
            "assets": [
                {"name": "leaf-x86_64-unknown-linux-gnu.tar.xz",
                 "browser_download_url": "https://example.test/a.tar.xz"},
                {"name": "dist-manifest.json",
                 "browser_download_url": "https://example.test/m.json"}
            ]
        }"#;
        let rel = parse_release(json).unwrap();
        assert_eq!(rel.version, Version::new(0, 9, 0));
        assert_eq!(
            rel.assets.get("dist-manifest.json").map(String::as_str),
            Some("https://example.test/m.json")
        );
    }
}
