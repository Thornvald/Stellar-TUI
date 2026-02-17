use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    Standard,
    CleanRebuild,
}

/// Handle to a running build process.
pub struct BuildHandle {
    finished: Arc<AtomicBool>,
    success: Arc<AtomicBool>,
    cancel_flag: Arc<AtomicBool>,
}

impl BuildHandle {
    /// Non-blocking check: returns Some(success) if finished.
    pub fn try_finished(&self) -> Option<bool> {
        if self.finished.load(Ordering::Relaxed) {
            Some(self.success.load(Ordering::Relaxed))
        } else {
            None
        }
    }

    /// Signal the build to cancel.
    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }
}

/// Derive the editor target name from a .uproject path.
fn derive_editor_target(project_path: &str) -> Result<String, String> {
    let path = PathBuf::from(project_path);
    if !path.exists() {
        return Err(format!("Project file not found: {}", project_path));
    }
    let stem = path
        .file_stem()
        .ok_or_else(|| "Invalid project file name".to_string())?
        .to_string_lossy()
        .to_string();
    if stem.to_lowercase().ends_with("editor") {
        Ok(stem)
    } else {
        Ok(format!("{}Editor", stem))
    }
}

/// Spawn a build as a background tokio task.
/// Log lines are sent through `tx`. Returns a handle to check status / cancel.
pub fn spawn_build(
    project_path: String,
    engine_path: String,
    tx: mpsc::UnboundedSender<String>,
    mode: BuildMode,
) -> Result<BuildHandle, String> {
    let ubt_dll = PathBuf::from(&engine_path)
        .join("Engine/Binaries/DotNET/UnrealBuildTool/UnrealBuildTool.dll");

    if !ubt_dll.exists() {
        return Err(format!(
            "UnrealBuildTool not found at {}",
            ubt_dll.display()
        ));
    }

    let target_name = derive_editor_target(&project_path)?;
    let project_dir = PathBuf::from(&project_path)
        .parent()
        .map(|p| p.to_path_buf());

    let cmd_display = match mode {
        BuildMode::Standard => format!(
            "dotnet \"{}\" {} Win64 Development -Project=\"{}\" -WaitMutex",
            ubt_dll.display(),
            target_name,
            project_path
        ),
        BuildMode::CleanRebuild => format!(
            "Clean Rebuild -> clean temp files, regenerate project files, then: dotnet \"{}\" {} Win64 Development -Project=\"{}\" -WaitMutex",
            ubt_dll.display(),
            target_name,
            project_path
        ),
    };
    let _ = tx.send(format!("Running: {}", cmd_display));

    let finished = Arc::new(AtomicBool::new(false));
    let success = Arc::new(AtomicBool::new(false));
    let cancel_flag = Arc::new(AtomicBool::new(false));

    let handle = BuildHandle {
        finished: finished.clone(),
        success: success.clone(),
        cancel_flag: cancel_flag.clone(),
    };

    tokio::spawn(async move {
        let result = run_build_process(
            &ubt_dll,
            &target_name,
            &project_path,
            project_dir.as_ref(),
            tx.clone(),
            cancel_flag,
            mode,
        )
        .await;

        match result {
            Ok(exit_success) => {
                success.store(exit_success, Ordering::Relaxed);
            }
            Err(e) => {
                let _ = tx.send(format!("Build error: {}", e));
                success.store(false, Ordering::Relaxed);
            }
        }
        finished.store(true, Ordering::Relaxed);
    });

    Ok(handle)
}

async fn run_build_process(
    ubt_dll: &PathBuf,
    target_name: &str,
    project_path: &str,
    project_dir: Option<&PathBuf>,
    tx: mpsc::UnboundedSender<String>,
    cancel_flag: Arc<AtomicBool>,
    mode: BuildMode,
) -> Result<bool, String> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    if mode == BuildMode::CleanRebuild {
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = tx.send("Clean rebuild cancelled before starting.".to_string());
            return Ok(false);
        }

        let _ = tx.send("Clean rebuild: removing temporary project files...".to_string());
        clean_project_artifacts(project_path, project_dir, &tx).await?;

        if cancel_flag.load(Ordering::Relaxed) {
            let _ = tx.send("Clean rebuild cancelled before project file generation.".to_string());
            return Ok(false);
        }

        let _ = tx.send("Clean rebuild: regenerating project files...".to_string());
        regenerate_project_files(ubt_dll, project_path, project_dir, &tx).await?;
    }

    let mut cmd = Command::new("dotnet");
    cmd.arg(ubt_dll)
        .arg(target_name)
        .arg("Win64")
        .arg("Development")
        .arg(format!("-Project={}", project_path))
        .arg("-WaitMutex")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    if let Some(dir) = project_dir {
        cmd.current_dir(dir);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn dotnet: {}", e))?;

    // Stream stdout
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let tx_out = tx.clone();
    let tx_err = tx.clone();

    let stdout_task = tokio::spawn(async move {
        if let Some(stdout) = stdout {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = tx_out.send(line);
            }
        }
    });

    let stderr_task = tokio::spawn(async move {
        if let Some(stderr) = stderr {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = tx_err.send(line);
            }
        }
    });

    // Poll for completion or cancellation
    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = child.kill().await;
            let _ = tx.send("Build process killed.".to_string());
            stdout_task.abort();
            stderr_task.abort();
            return Ok(false);
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                // Wait for output tasks to finish draining
                let _ = stdout_task.await;
                let _ = stderr_task.await;
                return Ok(status.success());
            }
            Ok(None) => {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            Err(e) => {
                return Err(format!("Error waiting for process: {}", e));
            }
        }
    }
}

async fn clean_project_artifacts(
    project_path: &str,
    project_dir: Option<&PathBuf>,
    tx: &mpsc::UnboundedSender<String>,
) -> Result<(), String> {
    let Some(project_dir) = project_dir else {
        return Err("Could not determine project directory for clean rebuild.".to_string());
    };

    let dirs_to_remove = ["Binaries", "Intermediate", "Saved", ".vs"];
    for dir_name in dirs_to_remove {
        let full = project_dir.join(dir_name);
        if full.exists() {
            let _ = tx.send(format!("Removing directory: {}", full.display()));
            tokio::fs::remove_dir_all(&full)
                .await
                .map_err(|e| format!("Failed to remove {}: {}", full.display(), e))?;
        }
    }

    let project_file = PathBuf::from(project_path);
    let sln_from_project = project_file.with_extension("sln");
    let mut files_to_remove = vec![sln_from_project];

    if let Some(stem) = project_file.file_stem().map(|s| s.to_string_lossy().to_string()) {
        files_to_remove.push(project_dir.join(format!("{}.sln", stem)));
    }

    for file in files_to_remove {
        if file.exists() {
            let _ = tx.send(format!("Removing file: {}", file.display()));
            tokio::fs::remove_file(&file)
                .await
                .map_err(|e| format!("Failed to remove {}: {}", file.display(), e))?;
        }
    }

    Ok(())
}

async fn regenerate_project_files(
    ubt_dll: &PathBuf,
    project_path: &str,
    project_dir: Option<&PathBuf>,
    tx: &mpsc::UnboundedSender<String>,
) -> Result<(), String> {
    use tokio::process::Command;

    let mut cmd = Command::new("dotnet");
    cmd.arg(ubt_dll)
        .arg("-ProjectFiles")
        .arg(format!("-Project={}", project_path))
        .arg("-Game")
        .arg("-Engine");

    if let Some(dir) = project_dir {
        cmd.current_dir(dir);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to regenerate project files: {}", e))?;

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if !line.trim().is_empty() {
            let _ = tx.send(line.to_string());
        }
    }

    for line in String::from_utf8_lossy(&output.stderr).lines() {
        if !line.trim().is_empty() {
            let _ = tx.send(line.to_string());
        }
    }

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Project file generation failed with status: {}",
            output.status
        ))
    }
}
