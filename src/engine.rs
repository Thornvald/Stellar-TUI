use crate::types::EngineInstall;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Scan for Unreal Engine installations (ported from the Tauri backend).
pub fn detect_engines() -> Vec<EngineInstall> {
    let mut installs = Vec::new();
    let mut seen = HashSet::new();

    let mut base_dirs = Vec::new();

    #[cfg(windows)]
    {
        if let Ok(pf) = std::env::var("PROGRAMFILES") {
            base_dirs.push(PathBuf::from(pf).join("Epic Games"));
        }
        if let Ok(pf86) = std::env::var("PROGRAMFILES(X86)") {
            base_dirs.push(PathBuf::from(pf86).join("Epic Games"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        base_dirs.push(PathBuf::from("/Users/Shared/Epic Games"));
        if let Ok(home) = std::env::var("HOME") {
            base_dirs.push(PathBuf::from(home).join("Epic Games"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            base_dirs.push(PathBuf::from(&home).join("Epic Games"));
            base_dirs.push(PathBuf::from(&home).join(".local/share/Epic Games"));
        }
        base_dirs.push(PathBuf::from("/opt/Epic Games"));
    }

    // Scan base directories
    for base_dir in &base_dirs {
        if let Ok(entries) = fs::read_dir(base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if should_skip_directory(&name) {
                    continue;
                }
                let normalized = path.to_string_lossy().to_string();
                if seen.contains(&normalized) {
                    continue;
                }
                if !is_engine_root(&path) {
                    continue;
                }
                let version = parse_version_from_name(&name);
                let label = format_label(&name, &version);
                installs.push(EngineInstall {
                    id: normalized.clone(),
                    name: label,
                    path: normalized.clone(),
                    version,
                });
                seen.insert(normalized);
            }
        }
    }

    // Check Windows launcher installed file
    #[cfg(windows)]
    {
        let program_data = std::env::var("ProgramData")
            .unwrap_or_else(|_| "C:\\ProgramData".to_string());
        let launcher_paths = [
            PathBuf::from(&program_data)
                .join("Epic/UnrealEngineLauncher/LauncherInstalled.dat"),
            PathBuf::from(&program_data)
                .join("Epic/EpicGamesLauncher/LauncherInstalled.dat"),
        ];

        for launcher_path in &launcher_paths {
            if let Ok(contents) = fs::read_to_string(launcher_path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&contents) {
                    if let Some(list) =
                        data.get("InstallationList").and_then(|v| v.as_array())
                    {
                        for item in list {
                            if let Some(location) =
                                item.get("InstallLocation").and_then(|v| v.as_str())
                            {
                                let path = PathBuf::from(location);
                                let name = path
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_default();
                                if should_skip_directory(&name) {
                                    continue;
                                }
                                let normalized = path.to_string_lossy().to_string();
                                if seen.contains(&normalized) {
                                    continue;
                                }
                                if !is_engine_root(&path) {
                                    continue;
                                }
                                let version = item
                                    .get("AppVersion")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .or_else(|| parse_version_from_name(&name));
                                let display_name = item
                                    .get("DisplayName")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(&name);
                                let label = format_label(display_name, &version);
                                installs.push(EngineInstall {
                                    id: normalized.clone(),
                                    name: label,
                                    path: normalized.clone(),
                                    version,
                                });
                                seen.insert(normalized);
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by version descending
    installs.sort_by(|a, b| {
        match (&b.version, &a.version) {
            (Some(bv), Some(av)) => {
                let a_parts: Vec<u32> =
                    av.split('.').filter_map(|s| s.parse().ok()).collect();
                let b_parts: Vec<u32> =
                    bv.split('.').filter_map(|s| s.parse().ok()).collect();
                b_parts.cmp(&a_parts)
            }
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });

    installs
}

fn parse_version_from_name(name: &str) -> Option<String> {
    if let Some(caps) = regex::Regex::new(r"(?i)UE[_-]([0-9]+(?:\.[0-9]+)*)")
        .ok()
        .and_then(|re| re.captures(name))
    {
        return caps.get(1).map(|m| m.as_str().to_string());
    }
    if let Some(caps) = regex::Regex::new(r"([0-9]+(?:\.[0-9]+)*)")
        .ok()
        .and_then(|re| re.captures(name))
    {
        return caps.get(1).map(|m| m.as_str().to_string());
    }
    None
}

fn should_skip_directory(name: &str) -> bool {
    let skip = [
        "launcher",
        "epicgameslauncher",
        "epic games launcher",
        "epic online services",
        "directxredist",
        "vcredist",
    ];
    let lower = name.to_lowercase();
    skip.iter().any(|s| lower == *s)
}

fn is_engine_root(path: &PathBuf) -> bool {
    let engine_dir = path.join("Engine");
    if !engine_dir.is_dir() {
        return false;
    }
    engine_dir.join("Binaries").is_dir() || engine_dir.join("Build").is_dir()
}

fn format_label(name: &str, version: &Option<String>) -> String {
    if let Some(v) = version {
        format!("Unreal Engine {}", v)
    } else if !name.is_empty() {
        format!("Unreal Engine ({})", name)
    } else {
        "Unreal Engine".to_string()
    }
}
