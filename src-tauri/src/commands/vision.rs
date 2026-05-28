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

    // Estrai store da raw_text: prima riga che sia prevalentemente maiuscola (merchant bancario)
    // Escludi righe troppo corte, solo numeri, o parole comuni di UI
    let ui_noise = ["fehu","dashboard","transazioni","categorie","dati","foto","obiettivi",
                    "uscita","entrata","importo","descrizione","categoria","metodo","contanti","carta",
                    "pagamento","shopping","servizi","altro"];
    for line in raw_text.lines() {
        let trimmed = line.trim();
        if trimmed.len() < 3 { continue; }
        let low = trimmed.to_lowercase();
        if ui_noise.iter().any(|w| low.contains(w)) { continue; }
        // conta caratteri uppercase vs totali (lettere)
        let letters: usize = trimmed.chars().filter(|c| c.is_alphabetic()).count();
        let uppers: usize = trimmed.chars().filter(|c| c.is_uppercase()).count();
        if letters > 2 && uppers * 100 / letters.max(1) >= 60 {
            // prendi solo token uppercase-heavy (stoppa alla prima parola mista come "Data")
            let merchant_tokens: Vec<&str> = trimmed.split_whitespace()
                .take_while(|tok| {
                    let l = tok.chars().filter(|c| c.is_alphabetic()).count();
                    let u = tok.chars().filter(|c| c.is_uppercase()).count();
                    l == 0 || u * 100 / l.max(1) >= 60
                })
                .collect();
            let clean = merchant_tokens.join(" ");
            let clean = clean.trim_matches(|c: char| !c.is_alphanumeric() && c != '*');
            if !clean.is_empty() {
                data.descrizione = Some(clean.to_string());
                break;
            }
        }
    }

    // Step 2: qwen2.5-coder — solo categoria (prompt minimal: risposta = 1 parola)
    let merchant = data.descrizione.as_deref().unwrap_or("unknown transaction");
    let client = reqwest::Client::new();
    let parse_prompt = format!(
        "Reply with ONE word only from this list: Cibo, Trasporti, Casa, Salute, Svago, Abbigliamento, Istruzione, Sport, Lavoro, Altro.\n\
Cibo=food/restaurants; Trasporti=transport/fuel/flights; Casa=utilities/rent/home; Salute=pharmacy/medical; \
Svago=streaming/cinema/games; Abbigliamento=clothing; Istruzione=education/books; Sport=gym/sports; \
Lavoro=hosting/cloud/software/domains/Cloudflare/AWS/Adobe/professional; Altro=everything else.\n\
Transaction: {merchant}\nCategory:"
    );

    let parse_body = serde_json::json!({
        "model": "qwen2.5-coder:7b",
        "prompt": parse_prompt,
        "stream": false,
        "keep_alive": 0
    });

    if let Ok(resp) = client
        .post("http://localhost:11434/api/generate")
        .json(&parse_body)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            let word = json["response"].as_str().unwrap_or("").trim()
                .split_whitespace().next().unwrap_or("")
                .trim_matches(|c: char| !c.is_alphabetic())
                .to_lowercase();
            let valid = [("cibo","Cibo"),("trasporti","Trasporti"),("casa","Casa"),
                         ("salute","Salute"),("svago","Svago"),("abbigliamento","Abbigliamento"),
                         ("istruzione","Istruzione"),("sport","Sport"),("lavoro","Lavoro"),("altro","Altro")];
            for (key, display) in valid {
                if word == key {
                    data.categoria = Some(display.to_string());
                    break;
                }
            }
        }
    }

    Ok(data)
}
