use crate::error::AppError;
use serde::Serialize;

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
