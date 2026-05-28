use crate::{error::{AppError, AppResult}, models::ReceiptData};
use base64::{engine::general_purpose::STANDARD, Engine};

#[tauri::command]
pub async fn read_image_base64(path: String) -> AppResult<String> {
    let bytes = std::fs::read(&path)
        .map_err(|e| AppError::Validation(format!("Impossibile leggere il file: {e}")))?;
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "image/jpeg",
    };
    Ok(format!("data:{};base64,{}", mime, STANDARD.encode(&bytes)))
}

#[tauri::command]
pub async fn analyze_receipt(image_path: String) -> AppResult<ReceiptData> {
    let bytes = std::fs::read(&image_path)
        .map_err(|e| AppError::Validation(format!("Impossibile leggere il file: {e}")))?;
    let b64 = STANDARD.encode(&bytes);

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "moondream",
        "prompt": "Look at this receipt image. Extract these 4 values and list them one per line:\nAMOUNT: [total amount as number, e.g. 12.50]\nDATE: [date as YYYY-MM-DD, e.g. 2024-01-15]\nSTORE: [store or restaurant name]\nCATEGORY: [one of: Cibo, Trasporti, Casa, Salute, Svago, Abbigliamento, Istruzione, Sport, Lavoro, Altro]\nIf you cannot read a value write N/A.",
        "images": [b64],
        "stream": false
    });

    let resp = client
        .post("http://localhost:11434/api/generate")
        .json(&body)
        .timeout(std::time::Duration::from_secs(90))
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Errore Ollama (moondream non installato?): {e}")))?;

    let ollama: serde_json::Value = resp.json().await
        .map_err(|e| AppError::Validation(format!("Risposta Ollama non valida: {e}")))?;

    let text = ollama["response"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Campo response mancante".into()))?;

    // Parse key: value lines — robust to extra text
    let mut data = ReceiptData::default();
    for line in text.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("AMOUNT:").or_else(|| line.strip_prefix("Amount:")) {
            let val = val.trim().replace(',', ".").replace(['€','$',' '], "");
            if let Ok(f) = val.parse::<f64>() { data.importo = Some(f); }
        } else if let Some(val) = line.strip_prefix("DATE:").or_else(|| line.strip_prefix("Date:")) {
            let val = val.trim();
            if val != "N/A" && !val.is_empty() { data.data = Some(val.to_string()); }
        } else if let Some(val) = line.strip_prefix("STORE:").or_else(|| line.strip_prefix("Store:")) {
            let val = val.trim();
            if val != "N/A" && !val.is_empty() { data.descrizione = Some(val.to_string()); }
        } else if let Some(val) = line.strip_prefix("CATEGORY:").or_else(|| line.strip_prefix("Category:")) {
            let val = val.trim();
            if val != "N/A" && !val.is_empty() { data.categoria = Some(val.to_string()); }
        }
    }

    // Fallback: try to find any number if amount still missing
    if data.importo.is_none() {
        let re = regex::Regex::new(r"(?:total|totale|importo)[^\d]*(\d+[.,]\d{1,2})").ok();
        if let Some(re) = re {
            if let Some(cap) = re.captures(&text.to_lowercase()) {
                if let Ok(f) = cap[1].replace(',', ".").parse::<f64>() {
                    data.importo = Some(f);
                }
            }
        }
    }

    Ok(data)
}
