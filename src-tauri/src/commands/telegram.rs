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

    // Prefer venv python (sidecar/.venv/bin/python3) if available
    let sidecar_dir = script.parent().expect("script has parent");
    let venv_python = sidecar_dir.join(".venv/bin/python3");
    let python = if venv_python.exists() {
        venv_python
    } else {
        std::path::PathBuf::from("python3")
    };

    // Verify Python dependencies before spawning (also validates aiogram 3+ FSM API)
    let dep_check = std::process::Command::new(&python)
        .args(["-c", "import aiogram, aiosqlite; from aiogram.fsm.context import FSMContext"])
        .output();
    match dep_check {
        Ok(out) if !out.status.success() => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let first_line = stderr.lines().next().unwrap_or("").to_string();
            return Err(AppError::Validation(format!(
                "Dipendenze Python mancanti. Esegui:\n\ncd sidecar && python3 -m venv .venv && .venv/bin/pip install aiogram aiosqlite\n\nErrore: {}",
                first_line
            )));
        }
        Err(e) => {
            return Err(AppError::Validation(format!(
                "Python non trovato: {e}"
            )));
        }
        Ok(_) => {}
    }

    // Kill tracked process + any zombie fehu_bot.py processes from previous sessions.
    // Use SIGTERM first so aiogram can close the Telegram polling session cleanly,
    // then SIGKILL after a delay to ensure stragglers are gone.
    {
        let mut guard = state.bot.lock().unwrap();
        if let Some(child) = guard.as_mut() {
            // Send SIGTERM via pkill below; just remove from tracking
            let _ = child;
        }
        *guard = None;
    }
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let _ = std::process::Command::new("pkill").args(["-f", "fehu_bot.py"]).output(); // SIGTERM
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let _ = std::process::Command::new("pkill").args(["-9", "-f", "fehu_bot.py"]).output(); // SIGKILL stragglers
    }
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("wmic")
        .args(["process", "where", "commandline like '%fehu_bot.py%'", "call", "terminate"])
        .output();
    // Give Telegram API time to release the polling session
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Spawn Python bot, redirect stdout+stderr to log file
    let mut cmd = std::process::Command::new(&python);
    cmd.arg(&script)
        .arg("--token").arg(&token)
        .arg("--db-path").arg(db_path.to_str().unwrap_or(""));
    if let Ok(log) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/fehu_bot_app.log") {
        if let Ok(log2) = log.try_clone() {
            cmd.stdout(log).stderr(log2);
        }
    }
    let child = cmd.spawn()
        .map_err(|e| AppError::Validation(
            format!("Impossibile avviare python: {e}")
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
