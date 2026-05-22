use std::fs::{self, File};
use std::io::copy;
use std::path::{Path, PathBuf};
use std::time::Duration;

use base64::Engine;
use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::models::{AddonInfo, Settings};

// --- GitHub-hosted add-on (Kneuma's own — installs from GitHub Releases) ---

struct GithubEntry {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    folder: &'static str,
    toc: &'static str,
    repo: &'static str,
    image_url: &'static str,
}

const GITHUB: &[GithubEntry] = &[GithubEntry {
    id: "mop_gm",
    name: "MoP_GM — GM Panel",
    description: "A clean in-game panel for the most-used GM commands (spawn, teleport, items, cheats). Made by Kneuma — recommended for any GM.",
    folder: "MoP_GM",
    toc: "MoP_GM.toc",
    repo: "timoinglin/MoP_GM",
    image_url: "https://raw.githubusercontent.com/timoinglin/MoP_GM/main/screenshots/guild.png",
}];

// --- Bundled add-ons (shipped with Aegis; defined in resources/addons/catalog.json) ---

#[derive(Deserialize)]
struct BundledEntry {
    id: String,
    name: String,
    description: String,
    folder: String,
    zip: String,
    thumbnail: String,
}

#[derive(Deserialize)]
struct Catalog {
    addons: Vec<BundledEntry>,
}

/// The bundled add-ons folder: shipped under resource_dir in a release, or the
/// source folder in dev.
fn bundled_dir(app: &AppHandle) -> Option<PathBuf> {
    // Dev/source first — this compile-time path only exists on the build machine,
    // and is always the freshest copy while developing.
    let dev = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources").join("addons");
    if dev.join("catalog.json").is_file() {
        return Some(dev);
    }
    // Production: bundled next to the executable.
    if let Ok(rd) = app.path().resource_dir() {
        let p = rd.join("resources").join("addons");
        if p.join("catalog.json").is_file() {
            return Some(p);
        }
    }
    None
}

fn load_catalog(app: &AppHandle) -> Vec<BundledEntry> {
    let Some(dir) = bundled_dir(app) else { return Vec::new() };
    fs::read_to_string(dir.join("catalog.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<Catalog>(&t).ok())
        .map(|c| c.addons)
        .unwrap_or_default()
}

// --- Shared helpers ---

fn addons_dir(settings: &Settings) -> Option<PathBuf> {
    let client = settings.client_path.as_deref()?;
    Some(Path::new(client).join("Interface").join("AddOns"))
}

fn installed_version(addons: &Path, folder: &str, toc: &str) -> Option<String> {
    let text = fs::read_to_string(addons.join(folder).join(toc)).ok()?;
    text.lines()
        .find_map(|l| l.trim().strip_prefix("## Version:").map(|r| r.trim().to_string()))
}

fn latest_version(repo: &str) -> Option<String> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let resp = ureq::get(&url)
        .set("User-Agent", "Aegis")
        .set("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(10))
        .call()
        .ok()?;
    let json: serde_json::Value = resp.into_json().ok()?;
    Some(json.get("tag_name")?.as_str()?.trim_start_matches('v').to_string())
}

pub fn status(app: &AppHandle, settings: &Settings) -> Vec<AddonInfo> {
    let addons = addons_dir(settings);
    let mut out = Vec::new();

    // GitHub add-ons.
    for e in GITHUB {
        let installed_version = addons.as_deref().and_then(|d| installed_version(d, e.folder, e.toc));
        let latest = latest_version(e.repo);
        let update_available = matches!((&installed_version, &latest), (Some(c), Some(n)) if c != n);
        out.push(AddonInfo {
            id: e.id.into(),
            name: e.name.into(),
            description: e.description.into(),
            installed: installed_version.is_some(),
            installed_version,
            latest_version: latest,
            update_available,
            has_thumbnail: false,
            featured: true,
            image_url: Some(e.image_url.into()),
        });
    }

    // Bundled add-ons — only shown when their zip is actually present.
    let dir = bundled_dir(app);
    for e in load_catalog(app) {
        let zip_present = dir.as_ref().map(|d| d.join(&e.zip).is_file()).unwrap_or(false);
        if !zip_present {
            continue;
        }
        let installed = addons.as_deref().map(|d| d.join(&e.folder).is_dir()).unwrap_or(false);
        let has_thumbnail = dir.as_ref().map(|d| d.join(&e.thumbnail).is_file()).unwrap_or(false);
        out.push(AddonInfo {
            id: e.id,
            name: e.name,
            description: e.description,
            installed,
            installed_version: None,
            latest_version: None,
            update_available: false,
            has_thumbnail,
            featured: false,
            image_url: None,
        });
    }
    out
}

/// Base64 data-URI of a bundled add-on's thumbnail (for display in the webview).
pub fn thumbnail(app: &AppHandle, id: &str) -> Option<String> {
    let e = load_catalog(app).into_iter().find(|x| x.id == id)?;
    let bytes = fs::read(bundled_dir(app)?.join(&e.thumbnail)).ok()?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Some(format!("data:image/png;base64,{b64}"))
}

pub fn install(app: &AppHandle, settings: &Settings, id: &str) -> Result<String, String> {
    if GITHUB.iter().any(|e| e.id == id) {
        return install_github(settings, id);
    }
    install_bundled(app, settings, id)
}

fn install_bundled(app: &AppHandle, settings: &Settings, id: &str) -> Result<String, String> {
    let e = load_catalog(app).into_iter().find(|x| x.id == id).ok_or("Unknown add-on.")?;
    let dir = bundled_dir(app).ok_or("Bundled add-ons aren't available in this build.")?;
    let zip_path = dir.join(&e.zip);
    if !zip_path.is_file() {
        return Err("That add-on isn't included in this build.".into());
    }
    let addons = addons_dir(settings).ok_or("Set your WoW client folder in Settings first.")?;
    if !addons.is_dir() {
        return Err(format!("Couldn't find your AddOns folder at {}.", addons.display()));
    }

    let file = File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Bad zip: {e}"))?;
    // Clean reinstall of the addon's own folder.
    let target = addons.join(&e.folder);
    if target.exists() {
        let _ = fs::remove_dir_all(&target);
    }
    archive.extract(&addons).map_err(|err| format!("Couldn't extract: {err}"))?;
    Ok(format!("Installed {}. Restart WoW (or type /reload) to see it.", e.name))
}

// --- GitHub install (download + extract, strip the wrapper folder) ---

fn install_github(settings: &Settings, id: &str) -> Result<String, String> {
    let e = GITHUB.iter().find(|x| x.id == id).ok_or("Unknown add-on.")?;
    let addons = addons_dir(settings).ok_or("Set your WoW client folder in Settings first.")?;
    if !addons.is_dir() {
        return Err(format!("Couldn't find your AddOns folder at {}.", addons.display()));
    }
    let tag = latest_version(e.repo).ok_or("Couldn't find the latest version on GitHub (are you online?).")?;
    let url = format!("https://github.com/{}/archive/refs/tags/v{tag}.zip", e.repo);

    let zip_path = std::env::temp_dir().join(format!("aegis-{}-{tag}.zip", e.id));
    download(&url, &zip_path)?;

    let result = (|| -> Result<String, String> {
        let file = File::open(&zip_path).map_err(|err| err.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|err| format!("Bad zip: {err}"))?;
        let wrapper = archive
            .by_index(0)
            .map_err(|err| err.to_string())?
            .name()
            .split('/')
            .next()
            .unwrap_or("")
            .to_string();
        if wrapper.is_empty() {
            return Err("Unexpected archive layout.".into());
        }
        archive.extract(&addons).map_err(|err| format!("Couldn't extract: {err}"))?;
        let extracted = addons.join(&wrapper);
        let target = addons.join(e.folder);
        if target.exists() {
            fs::remove_dir_all(&target).map_err(|err| format!("Couldn't replace the old version: {err}"))?;
        }
        fs::rename(&extracted, &target).map_err(|err| format!("Couldn't move the add-on into place: {err}"))?;
        let version = installed_version(&addons, e.folder, e.toc).unwrap_or(tag.clone());
        Ok(format!("Installed {} {version}. Restart WoW (or /reload) to see it.", e.name))
    })();

    let _ = fs::remove_file(&zip_path);
    result
}

fn download(url: &str, dest: &Path) -> Result<(), String> {
    let resp = ureq::get(url)
        .set("User-Agent", "Aegis")
        .timeout(Duration::from_secs(60))
        .call()
        .map_err(|e| format!("Download failed: {e}"))?;
    let mut reader = resp.into_reader();
    let mut file = File::create(dest).map_err(|e| format!("Couldn't save the download: {e}"))?;
    copy(&mut reader, &mut file).map_err(|e| format!("Download interrupted: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Catalog;

    #[test]
    fn github_catalog_has_mop_gm() {
        assert!(super::GITHUB.iter().any(|e| e.id == "mop_gm" && e.folder == "MoP_GM"));
    }

    #[test]
    fn bundled_catalog_json_parses() {
        // Tests run with the crate root as CWD.
        let text = std::fs::read_to_string("resources/addons/catalog.json").unwrap();
        let catalog: Catalog = serde_json::from_str(&text).unwrap();
        assert!(catalog.addons.len() >= 10);
        assert!(catalog.addons.iter().any(|a| a.id == "dbm"));
    }
}
