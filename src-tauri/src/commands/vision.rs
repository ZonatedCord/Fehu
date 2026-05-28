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

    let mut data = ReceiptData::default();

    // Estrai importo con regex Rust — più affidabile di LLM su pattern numerici
    // Prova: -9,40€ | -9.40€ | -9 40€ (spazio da OCR) | 9,40 € ecc.
    let re_amt = regex::Regex::new(r"-?(\d+)[,. ](\d{2})\s*[€$]").ok();
    if let Some(re) = re_amt {
        if let Some(cap) = re.captures(&raw_text) {
            let s = format!("{}.{}", &cap[1], &cap[2]);
            if let Ok(f) = s.parse::<f64>() { data.importo = Some(f); }
        }
    }

    // Estrai data con regex Rust — converte mesi italiani in YYYY-MM-DD
    let re_date = regex::Regex::new(
        r"(\d{1,2})\s+(Gen|Feb|Mar|Apr|Mag|Giu|Lug|Ago|Set|Ott|Nov|Dic)\s+(\d{4})"
    ).ok();
    if let Some(re) = re_date {
        if let Some(cap) = re.captures(&raw_text) {
            let day = format!("{:02}", cap[1].parse::<u32>().unwrap_or(1));
            let month = match &cap[2] {
                m if m.eq_ignore_ascii_case("gen") => "01",
                m if m.eq_ignore_ascii_case("feb") => "02",
                m if m.eq_ignore_ascii_case("mar") => "03",
                m if m.eq_ignore_ascii_case("apr") => "04",
                m if m.eq_ignore_ascii_case("mag") => "05",
                m if m.eq_ignore_ascii_case("giu") => "06",
                m if m.eq_ignore_ascii_case("lug") => "07",
                m if m.eq_ignore_ascii_case("ago") => "08",
                m if m.eq_ignore_ascii_case("set") => "09",
                m if m.eq_ignore_ascii_case("ott") => "10",
                m if m.eq_ignore_ascii_case("nov") => "11",
                _ => "12",
            };
            data.data = Some(format!("{}-{}-{}", &cap[3], month, day));
        }
    }
    // Fallback: data già in formato YYYY-MM-DD o DD/MM/YYYY
    if data.data.is_none() {
        if let Some(re) = regex::Regex::new(r"(\d{4})-(\d{2})-(\d{2})").ok() {
            if let Some(cap) = re.captures(&raw_text) {
                data.data = Some(format!("{}-{}-{}", &cap[1], &cap[2], &cap[3]));
            }
        }
    }
    if data.data.is_none() {
        if let Some(re) = regex::Regex::new(r"(\d{2})/(\d{2})/(\d{4})").ok() {
            if let Some(cap) = re.captures(&raw_text) {
                data.data = Some(format!("{}-{}-{}", &cap[3], &cap[2], &cap[1]));
            }
        }
    }

    // Step 2: qwen2.5-coder — solo store e categoria (semantica, non numeri)
    let client = reqwest::Client::new();
    let parse_prompt = format!(
        "You are a financial transaction categorizer. From this text extract:\n\
STORE: [merchant or payee name, one line only]\n\
CATEGORY: [pick exactly ONE Italian word: Cibo | Trasporti | Casa | Salute | Svago | Abbigliamento | Istruzione | Sport | Lavoro | Altro]\n\n\
Category guide — Cibo: food/restaurants/supermarkets; Trasporti: fuel/transport/flights; Casa: utilities/rent/furniture; Salute: pharmacy/medical; Svago: Netflix/Spotify/cinema/games; Abbigliamento: clothing/shoes; Istruzione: courses/books; Sport: gym/sports; Lavoro: hosting/software/cloud/Cloudflare/AWS/Adobe/PayPal-business; Altro: everything else.\n\n\
Output ONLY those 2 lines. No explanation.\n\nText:\n{raw_text}"
    );

    let parse_body = serde_json::json!({
        "model": "qwen2.5-coder:7b",
        "prompt": parse_prompt,
        "stream": false
    });

    if let Ok(resp) = client
        .post("http://localhost:11434/api/generate")
        .json(&parse_body)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
    {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            let text = json["response"].as_str().unwrap_or("");
            for line in text.lines() {
                // strip markdown bold/italic and normalise
                let clean = line.trim().trim_matches('*').trim_matches('_').trim();
                let low = clean.to_lowercase();
                if let Some(val) = low.strip_prefix("store:") {
                    let v = val.trim().trim_matches(|c: char| !c.is_alphanumeric() && c != ' ' && c != '*');
                    // use original-case from clean line
                    if !v.is_empty() && v != "n/a" && data.descrizione.is_none() {
                        let orig = &clean[clean.to_lowercase().find("store:").unwrap_or(0) + 6..];
                        let v2 = orig.trim().trim_matches(|c: char| !c.is_alphanumeric() && c != ' ' && c != '*');
                        if !v2.is_empty() { data.descrizione = Some(v2.to_string()); }
                    }
                } else if let Some(val) = low.strip_prefix("category:") {
                    let v = val.trim().trim_matches(|c: char| !c.is_alphanumeric());
                    let valid = ["cibo","trasporti","casa","salute","svago","abbigliamento","istruzione","sport","lavoro","altro"];
                    // find first valid category keyword in the value
                    for cat in valid {
                        if v.contains(cat) {
                            let display = match cat {
                                "cibo" => "Cibo", "trasporti" => "Trasporti", "casa" => "Casa",
                                "salute" => "Salute", "svago" => "Svago", "abbigliamento" => "Abbigliamento",
                                "istruzione" => "Istruzione", "sport" => "Sport", "lavoro" => "Lavoro",
                                _ => "Altro",
                            };
                            data.categoria = Some(display.to_string());
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(data)
}
