use crate::{
    error::{AppError, AppResult},
    AppState,
};
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn start_telegram_bot(
    app: AppHandle,
    state: State<'_, AppState>,
    token: String,
) -> AppResult<String> {
    if token.trim().is_empty() {
        return Err(AppError::Validation("Token bot non può essere vuoto".into()));
    }

    let db_path = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Validation(format!("App data dir: {e}")))?
        .join("fehu.db");

    // Locate the bot script across multiple candidate paths
    let script = {
        let from_resources = app
            .path()
            .resource_dir()
            .ok()
            .map(|p| p.join("sidecar/fehu_bot.py"));
        let from_cwd = std::env::current_dir()
            .ok()
            .map(|p| p.join("sidecar/fehu_bot.py"));
        // In Tauri dev, cwd is src-tauri; go up one level to project root
        let from_cwd_parent = std::env::current_dir()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("sidecar/fehu_bot.py")));
        // Compile-time project root (CARGO_MANIFEST_DIR = src-tauri, parent = project root)
        let from_manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .map(|p| p.join("sidecar/fehu_bot.py"));

        [from_resources, from_cwd, from_cwd_parent, from_manifest]
            .into_iter()
            .flatten()
            .find(|p| p.exists())
            .ok_or_else(|| AppError::Validation(
                "Script bot non trovato. Assicurati che sidecar/fehu_bot.py esista.".into(),
            ))?
    };

    // Verify Python dependencies before spawning
    let dep_check = std::process::Command::new("python3")
        .args(["-c", "import aiogram, aiosqlite"])
        .output();
    match dep_check {
        Ok(out) if !out.status.success() => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let first_line = stderr.lines().next().unwrap_or("").to_string();
            return Err(AppError::Validation(format!(
                "Dipendenze Python mancanti. Esegui nel terminale:\n\npip3 install aiogram aiosqlite\n\nErrore: {}",
                first_line
            )));
        }
        Err(e) => {
            return Err(AppError::Validation(format!(
                "python3 non trovato: {e}. Installa Python 3 e poi: pip3 install aiogram aiosqlite"
            )));
        }
        Ok(_) => {}
    }

    // Stop existing instance
    {
        let mut guard = state.bot.lock().unwrap();
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
        }
    }

    // Spawn Python bot
    let child = std::process::Command::new("python3")
        .arg(&script)
        .arg("--token")
        .arg(&token)
        .arg("--db-path")
        .arg(db_path.to_str().unwrap_or(""))
        .spawn()
        .map_err(|e| AppError::Validation(
            format!("Impossibile avviare python3: {e}")
        ))?;

    let pid = child.id();
    *state.bot.lock().unwrap() = Some(child);

    Ok(format!("Bot avviato (PID {pid})"))
}

#[tauri::command]
pub fn stop_telegram_bot(state: State<'_, AppState>) -> AppResult<()> {
    let mut guard = state.bot.lock().unwrap();
    if let Some(mut child) = guard.take() {
        child.kill().map_err(|e| AppError::Validation(format!("Stop bot: {e}")))?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_telegram_status(state: State<'_, AppState>) -> AppResult<bool> {
    let mut guard = state.bot.lock().unwrap();
    if let Some(child) = guard.as_mut() {
        // Check if still running
        match child.try_wait() {
            Ok(None) => return Ok(true),  // still running
            _ => { *guard = None; }       // exited
        }
    }
    Ok(false)
}
