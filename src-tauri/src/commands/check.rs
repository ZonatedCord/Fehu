use crate::error::AppError;
use serde::Serialize;

#[derive(Serialize)]
pub struct InstallResult {
    pub success: bool,
    pub output: String,
}

#[derive(Serialize)]
pub struct DepsStatus {
    pub tesseract: bool,
    pub ollama: bool,
    pub tesseract_version: Option<String>,
}

#[tauri::command]
pub async fn check_dependencies() -> Result<DepsStatus, AppError> {
    let mut tesseract = false;
    let mut tesseract_version: Option<String> = None;

    #[cfg(not(target_os = "windows"))]
    let candidates: &[&str] = &[
        "/opt/homebrew/bin/tesseract",
        "/usr/local/bin/tesseract",
        "/usr/bin/tesseract",
    ];
    #[cfg(target_os = "windows")]
    let candidates: &[&str] = &[
        "/opt/homebrew/bin/tesseract",
        "/usr/local/bin/tesseract",
        "/usr/bin/tesseract",
        r"C:\Program Files\Tesseract-OCR\tesseract.exe",
        r"C:\Program Files (x86)\Tesseract-OCR\tesseract.exe",
    ];

    for path in candidates {
        if std::path::Path::new(path).exists() {
            if let Ok(out) = std::process::Command::new(path).arg("--version").output() {
                tesseract = true;
                tesseract_version = Some(
                    String::from_utf8_lossy(&out.stdout)
                        .lines()
                        .next()
                        .unwrap_or("")
                        .to_string(),
                );
                break;
            }
        }
    }

    // PATH fallback
    if !tesseract {
        if let Ok(out) = std::process::Command::new("tesseract").arg("--version").output() {
            if out.status.success() {
                tesseract = true;
                tesseract_version = Some(
                    String::from_utf8_lossy(&out.stdout)
                        .lines()
                        .next()
                        .unwrap_or("")
                        .to_string(),
                );
            }
        }
    }

    let ollama = reqwest::Client::new()
        .get("http://localhost:11434/api/tags")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    Ok(DepsStatus { tesseract, ollama, tesseract_version })
}

#[tauri::command]
pub async fn install_dependency(dep: String) -> Result<InstallResult, AppError> {
    let (cmd, args): (&str, Vec<&str>) = match dep.as_str() {
        "tesseract" => {
            #[cfg(target_os = "macos")]
            { ("brew", vec!["install", "tesseract", "tesseract-lang"]) }
            #[cfg(not(target_os = "macos"))]
            { return Ok(InstallResult { success: false, output: "Installa Tesseract manualmente da https://github.com/UB-Mannheim/tesseract/wiki".into() }); }
        }
        "pip-bot" => {
            // PEP 668: macOS/Linux Homebrew Python blocks global installs.
            // Use --break-system-packages to explicitly override.
            #[cfg(target_os = "windows")]
            { ("pip3", vec!["install", "aiogram", "aiosqlite"]) }
            #[cfg(not(target_os = "windows"))]
            { ("pip3", vec!["install", "--break-system-packages", "aiogram", "aiosqlite"]) }
        }
        _ => return Ok(InstallResult { success: false, output: format!("Dipendenza sconosciuta: {dep}") }),
    };

    match std::process::Command::new(cmd).args(&args).output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let combined = format!("{stdout}{stderr}").trim().to_string();
            Ok(InstallResult { success: out.status.success(), output: if combined.is_empty() { "Completato.".into() } else { combined } })
        }
        Err(e) => Ok(InstallResult { success: false, output: format!("Errore: {e}") }),
    }
}
