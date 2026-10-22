use std::path::{Path, PathBuf};
use std::process::Command;

const GAME_EXECUTABLE_CANDIDATES: &[&str] = &[
    // Pokémon Essentials v21.1's normal playtest launcher.
    "Game.exe",
    // Common replacement runtimes, used only when the normal launcher is absent.
    "mkxp-z.exe",
    "mkxp.exe",
];

fn find_game_executable(project_path: &Path) -> Result<PathBuf, String> {
    if !project_path.is_dir() {
        return Err(format!(
            "Project folder does not exist or is not a directory: {}",
            project_path.display()
        ));
    }

    if !project_path.join("Game.rxproj").is_file() {
        return Err(format!(
            "Cannot launch this folder because Game.rxproj is missing: {}",
            project_path.display()
        ));
    }

    for candidate in GAME_EXECUTABLE_CANDIDATES {
        let executable = project_path.join(candidate);
        if executable.is_file() {
            return Ok(executable);
        }
    }

    Err(format!(
        "No playable game executable was found in {}. Checked: {}",
        project_path.display(),
        GAME_EXECUTABLE_CANDIDATES.join(", ")
    ))
}

#[tauri::command]
pub async fn launch_game(project_path: String) -> Result<String, String> {
    let project_path = PathBuf::from(&project_path)
        .canonicalize()
        .map_err(|e| format!("Could not access the project folder '{}': {e}", project_path))?;
    let executable = find_game_executable(&project_path)?
        .canonicalize()
        .map_err(|e| format!("Could not access the game executable: {e}"))?;

    // Use Command directly rather than a shell command. spawn() returns as soon
    // as the process starts, leaving Studio responsive while the game runs.
    Command::new(&executable)
        .current_dir(&project_path)
        .spawn()
        .map_err(|e| format!("Failed to launch '{}': {e}", executable.display()))?;

    let name = executable
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("game executable")
        .to_string();
    eprintln!("[playtest] Launched {} from {}", name, project_path.display());
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "essentials-studio-playtest-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    #[test]
    fn prefers_normal_essentials_game_executable() {
        let dir = test_dir("prefer-game");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Game.rxproj"), b"").unwrap();
        fs::write(dir.join("Game.exe"), b"").unwrap();
        fs::write(dir.join("mkxp-z.exe"), b"").unwrap();

        assert_eq!(find_game_executable(&dir).unwrap(), dir.join("Game.exe"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reports_missing_game_executable() {
        let dir = test_dir("missing-game");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Game.rxproj"), b"").unwrap();

        let error = find_game_executable(&dir).unwrap_err();
        assert!(error.contains("No playable game executable"));
        fs::remove_dir_all(dir).unwrap();
    }
}
