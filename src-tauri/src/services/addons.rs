use std::fs::{self, File};
use std::io::copy;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::models::{AddonInfo, Settings};

/// One entry in the add-on catalog. Extensible — add more entries to offer more.
struct CatalogEntry {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    /// AddOns subfolder + its .toc file (read for the installed version).
    folder: &'static str,
    toc: &'static str,
    /// GitHub "owner/repo" — releases drive the latest version + download.
    repo: &'static str,
}

const CATALOG: &[CatalogEntry] = &[CatalogEntry {
    id: "mop_gm",
    name: "MoP_GM — GM Panel",
    description: "A clean in-game panel for the most-used GM commands (spawn, teleport, items, cheats). Made by Kneuma.",
    folder: "MoP_GM",
    toc: "MoP_GM.toc",
    repo: "timoinglin/MoP_GM",
}];

fn entry(id: &str) -> Option<&'static CatalogEntry> {
    CATALOG.iter().find(|e| e.id == id)
}

/// <clientPath>\Interface\AddOns
fn addons_dir(settings: &Settings) -> Option<PathBuf> {
    let client = settings.client_path.as_deref()?;
    Some(Path::new(client).join("Interface").join("AddOns"))
}

/// Read `## Version:` from an installed addon's .toc.
fn installed_version(addons: &Path, e: &CatalogEntry) -> Option<String> {
    let toc = addons.join(e.folder).join(e.toc);
    let text = fs::read_to_string(toc).ok()?;
    for line in text.lines() {
        let l = line.trim();
        if let Some(rest) = l.strip_prefix("## Version:") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

/// Latest release tag from GitHub, with the leading `v` stripped (e.g. "1.2.0").
fn latest_version(repo: &str) -> Option<String> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let resp = ureq::get(&url)
        .set("User-Agent", "Aegis")
        .set("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(10))
        .call()
        .ok()?;
    let json: serde_json::Value = resp.into_json().ok()?;
    let tag = json.get("tag_name")?.as_str()?;
    Some(tag.trim_start_matches('v').to_string())
}

pub fn status(settings: &Settings) -> Vec<AddonInfo> {
    let addons = addons_dir(settings);
    CATALOG
        .iter()
        .map(|e| {
            let installed_version = addons.as_deref().and_then(|d| installed_version(d, e));
            let latest = latest_version(e.repo);
            let update_available = match (&installed_version, &latest) {
                (Some(cur), Some(new)) => cur != new,
                _ => false,
            };
            AddonInfo {
                id: e.id.to_string(),
                name: e.name.to_string(),
                description: e.description.to_string(),
                installed: installed_version.is_some(),
                installed_version,
                latest_version: latest,
                update_available,
            }
        })
        .collect()
}

/// Download the latest release and install it into the client's AddOns folder.
pub fn install(settings: &Settings, id: &str) -> Result<String, String> {
    let e = entry(id).ok_or("Unknown add-on.")?;
    let addons = addons_dir(settings)
        .ok_or("Set your WoW client folder in Settings first.")?;
    if !addons.is_dir() {
        return Err(format!(
            "Couldn't find your AddOns folder at {}. Check the client folder in Settings.",
            addons.display()
        ));
    }

    let tag = latest_version(e.repo)
        .ok_or("Couldn't find the latest version on GitHub (are you online?).")?;
    let url = format!("https://github.com/{}/archive/refs/tags/v{tag}.zip", e.repo);

    // Download the release zip to a temp file.
    let zip_path = std::env::temp_dir().join(format!("aegis-{}-{tag}.zip", e.id));
    download(&url, &zip_path)?;

    // Extract into the AddOns folder (same drive → cheap rename afterwards).
    let result = (|| -> Result<String, String> {
        let file = File::open(&zip_path).map_err(|err| err.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|err| format!("Bad zip: {err}"))?;

        // GitHub wraps everything in a single top folder, e.g. "MoP_GM-1.2.0/".
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

        let version = installed_version(&addons, e).unwrap_or(tag.clone());
        Ok(format!("Installed {} {version}. Restart WoW (or /reload) to see it.", e.name))
    })();

    let _ = fs::remove_file(&zip_path); // tidy up regardless
    result
}

/// Stream a URL to a file, following redirects (ureq does this by default).
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
    use super::*;

    #[test]
    fn catalog_has_mop_gm() {
        let e = entry("mop_gm").expect("mop_gm in catalog");
        assert_eq!(e.folder, "MoP_GM");
        assert_eq!(e.repo, "timoinglin/MoP_GM");
        assert!(entry("nope").is_none());
    }
}
