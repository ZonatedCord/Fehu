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

fn find_tesseract() -> Option<String> {
    for path in ["/opt/homebrew/bin/tesseract", "/usr/local/bin/tesseract", "/usr/bin/tesseract"] {
        if std::path::Path::new(path).exists() { return Some(path.to_string()); }
    }
    None
}

#[tauri::command]
pub async fn analyze_receipt(image_path: String) -> AppResult<ReceiptData> {
    let tess = find_tesseract()
        .ok_or_else(|| AppError::Validation("Tesseract non trovato. Installa con: brew install tesseract".into()))?;

    // Step 1: tesseract OCR — estrae testo grezzo (ita+eng, psm 6 = blocco testo uniforme)
    let output = std::process::Command::new(&tess)
        .args([image_path.as_str(), "stdout", "-l", "ita+eng", "--psm", "6", "--dpi", "150"])
        .output()
        .map_err(|e| AppError::Validation(format!("Errore avvio tesseract: {e}")))?;

    let raw_text = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::Validation(format!("Tesseract fallito: {stderr}")));
    };

    if raw_text.is_empty() {
        return Err(AppError::Validation("Nessun testo estratto dall'immagine".into()));
    }

    // Step 2: qwen2.5-coder — estrae struttura dal testo grezzo
    let client = reqwest::Client::new();
    let parse_prompt = format!(
        "You are a financial document parser. Given this raw text extracted from a receipt, invoice, or bank transaction screenshot, output ONLY these 4 lines (no other text):\nAMOUNT: [the transaction amount as positive decimal number, e.g. 9.40 — strip any minus sign or currency symbol]\nDATE: [date as YYYY-MM-DD, convert Italian month names: Gen=01 Feb=02 Mar=03 Apr=04 Mag=05 Giu=06 Lug=07 Ago=08 Set=09 Ott=10 Nov=11 Dic=12]\nSTORE: [merchant, payee, or business name]\nCATEGORY: [one of: Cibo, Trasporti, Casa, Salute, Svago, Abbigliamento, Istruzione, Sport, Lavoro, Altro]\n\nIf a value is not present, write N/A.\n\nDocument text:\n{raw_text}"
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
        // Prova prima con keyword (totale/importo/pagato)
        let re_kw = regex::Regex::new(r"(?:total[ei]?|importo|da pagare|pagato)[^\d]*(\d+[,. ]\d{1,2})").ok();
        if let Some(re) = re_kw {
            if let Some(cap) = re.captures(&raw_text.to_lowercase()) {
                let s = cap[1].replace([',', ' '], ".");
                if let Ok(f) = s.parse::<f64>() { data.importo = Some(f); }
            }
        }
    }
    if data.importo.is_none() {
        // Fallback generico: pattern -?N,NN€ o N NN€
        let re_amt = regex::Regex::new(r"-?(\d+)[,. ](\d{2})\s*[€$]").ok();
        if let Some(re) = re_amt {
            if let Some(cap) = re.captures(&raw_text) {
                let s = format!("{}.{}", &cap[1], &cap[2]);
                if let Ok(f) = s.parse::<f64>() { data.importo = Some(f); }
            }
        }
    }

    Ok(data)
}
