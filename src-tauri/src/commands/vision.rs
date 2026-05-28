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

    // Step 1: moondream — OCR grezzo sull'immagine
    let ocr_body = serde_json::json!({
        "model": "moondream",
        "prompt": "Read all the text visible in this receipt image. List every word and number you can see, including prices, dates, and store names. Be as complete as possible.",
        "images": [b64],
        "stream": false
    });

    let ocr_resp = client
        .post("http://localhost:11434/api/generate")
        .json(&ocr_body)
        .timeout(std::time::Duration::from_secs(90))
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Errore Ollama moondream: {e}")))?;

    let ocr_json: serde_json::Value = ocr_resp.json().await
        .map_err(|e| AppError::Validation(format!("Risposta moondream non valida: {e}")))?;

    let raw_text = ocr_json["response"].as_str().unwrap_or("").to_string();

    // Step 2: qwen2.5-coder — estrae struttura dal testo grezzo
    let parse_prompt = format!(
        "You are a receipt parser. Given this raw text extracted from a receipt, output ONLY these 4 lines (no other text):\nAMOUNT: [total amount paid as decimal number, e.g. 12.50]\nDATE: [date as YYYY-MM-DD]\nSTORE: [store or business name]\nCATEGORY: [one of: Cibo, Trasporti, Casa, Salute, Svago, Abbigliamento, Istruzione, Sport, Lavoro, Altro]\n\nIf a value is not present, write N/A.\n\nReceipt text:\n{raw_text}"
    );

    let parse_body = serde_json::json!({
        "model": "qwen2.5-coder:7b",
        "prompt": parse_prompt,
        "stream": false
    });

    let parse_resp = client
        .post("http://localhost:11434/api/generate")
        .json(&parse_body)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Errore Ollama qwen: {e}")))?;

    let parse_json: serde_json::Value = parse_resp.json().await
        .map_err(|e| AppError::Validation(format!("Risposta qwen non valida: {e}")))?;

    let text = parse_json["response"].as_str().unwrap_or(&raw_text);

    // Parse righe KEY: VALUE
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

    // Fallback regex sull'OCR grezzo se importo ancora mancante
    if data.importo.is_none() {
        let re = regex::Regex::new(r"(?:total[ei]?|importo|da pagare|pagato)[^\d]*(\d+[.,]\d{1,2})").ok();
        if let Some(re) = re {
            if let Some(cap) = re.captures(&raw_text.to_lowercase()) {
                if let Ok(f) = cap[1].replace(',', ".").parse::<f64>() {
                    data.importo = Some(f);
                }
            }
        }
    }

    Ok(data)
}
