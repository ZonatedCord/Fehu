use crate::{
    error::{AppError, AppResult},
    models::ReceiptData,
    AppState,
};
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

fn find_tesseract(override_path: &str) -> Option<String> {
    if !override_path.is_empty() && std::path::Path::new(override_path).exists() {
        return Some(override_path.to_string());
    }

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
            return Some(path.to_string());
        }
    }

    // PATH fallback — works if tesseract is on the system PATH
    if std::process::Command::new("tesseract")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("tesseract".to_string());
    }

    None
}

fn categorize_by_keywords(text: &str) -> Option<String> {
    let lower = text.to_lowercase();

    let rules: &[(&str, &[&str])] = &[
        ("Cibo", &[
            "esselunga","carrefour","conad","lidl","coop ","eurospin","aldi ","penny ","iper ",
            "pam ","unes ","naturasi","gros","bar ","caffè","caffe","pizza","ristorante",
            "trattoria","osteria","supermercato","alimentari","rosticceria","paninoteca",
            "mcdonald","burger","sushi","kebab","gelateria","pasticceria","bakery",
        ]),
        ("Trasporti", &[
            "trenitalia","italotreno","frecciarossa","ryanair","easyjet","wizzair",
            "lufthansa","vueling","ita airways","atm ","atac ","gtt ","seta ","autobus",
            "taxi","uber","blablacar","autostrada","telepass","q8 ","eni carb","ip carb",
            "agip","tamoil","esso ","parcheggio","parking","noleggio auto","hertz","avis","europcar",
        ]),
        ("Casa", &[
            "ikea","leroy merlin","brico","castorama","bricofer","obi ","affitto","condominio",
            "enel ","eni gas","a2a ","iren ","hera ","acquedotto","telecom","vodafone","tim ",
            "wind ","fastweb","sky ","fibra","luce ","gas ",
        ]),
        ("Salute", &[
            "farmacia","parafarmacia","medico","dentista","odontoiatra","ospedale","clinica",
            "laboratorio analisi","veterinario","ottica","fisioterapia",
        ]),
        ("Svago", &[
            "netflix","spotify","amazon prime","disney+","apple tv","hbo ","dazn ","twitch",
            "steam ","playstation","xbox ","nintendo","cinema","teatro","concerti",
            "ticketone","ticketmaster","booking.com","airbnb","tripadvisor",
        ]),
        ("Abbigliamento", &[
            "zara","h&m","primark","mango ","benetton","levi","nike ","adidas ","puma ",
            "decathlon","calzedonia","intimissimi","yamamay","coin ","rinascente",
        ]),
        ("Istruzione", &[
            "udemy","coursera","skillshare","masterclass","edx ","libreria","feltrinelli",
            "mondadori","rizzoli","ibs ","università","corso ","formazione",
        ]),
        ("Sport", &[
            "palestra","gym ","fitness","piscina","tennis","calcio","rugby","basket",
            "intersport","sportmaster",
        ]),
        ("Lavoro", &[
            "fattura","compenso","prestazione","consulenza","hetzner","ovh ","aruba ",
            "register.it","namecheap","cloudflare","aws ","digitalocean","linode",
            "adobe ","microsoft 365","office 365","slack ","notion ","figma ","github",
        ]),
    ];

    for (category, keywords) in rules {
        for kw in *keywords {
            if lower.contains(kw) {
                return Some(category.to_string());
            }
        }
    }
    None
}

#[tauri::command]
pub async fn analyze_receipt(
    state: tauri::State<'_, AppState>,
    image_path: String,
) -> AppResult<ReceiptData> {
    // Read settings synchronously; release lock before any async work
    let (tess_override, ollama_url, ollama_model) = {
        let db = state.db.0.lock().unwrap();
        let tess = db
            .query_row("SELECT value FROM settings WHERE key='tesseract_path'", [], |r| {
                r.get::<_, String>(0)
            })
            .unwrap_or_default();
        let url = db
            .query_row("SELECT value FROM settings WHERE key='ollama_url'", [], |r| {
                r.get::<_, String>(0)
            })
            .unwrap_or_else(|_| "http://localhost:11434".into());
        let model = db
            .query_row("SELECT value FROM settings WHERE key='ollama_model'", [], |r| {
                r.get::<_, String>(0)
            })
            .unwrap_or_else(|_| "qwen2.5-coder:7b".into());
        (tess, url, model)
    };

    let tess = find_tesseract(&tess_override)
        .ok_or_else(|| AppError::Validation(
            "Tesseract non trovato. Installa con: brew install tesseract tesseract-lang".into(),
        ))?;

    // Step 1: Tesseract OCR
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

    // Metodo da keywords pagamento
    {
        let lower = raw_text.to_lowercase();
        let card_kw = ["postepay", "bancomat", "carta", "pos", "mastercard", "visa", "amex", "paypal", "maestro", "contactless"];
        data.metodo = Some(if card_kw.iter().any(|k| lower.contains(k)) { "carta" } else { "contanti" }.to_string());
    }

    // Importo via regex Rust — handles: -9,40€ | -9.40€ | -9 40€ | 9,40 €
    let re_amt = regex::Regex::new(r"-?(\d+)[,. ](\d{2})\s*[€$]").ok();
    if let Some(re) = re_amt {
        if let Some(cap) = re.captures(&raw_text) {
            let s = format!("{}.{}", &cap[1], &cap[2]);
            if let Ok(f) = s.parse::<f64>() {
                data.importo = Some(f);
            }
        }
    }

    // Data con mesi italiani → YYYY-MM-DD
    let re_date = regex::Regex::new(
        r"(\d{1,2})\s+(Gen|Feb|Mar|Apr|Mag|Giu|Lug|Ago|Set|Ott|Nov|Dic)\s+(\d{4})",
    )
    .ok();
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

    // Store: prima riga uppercase-dominante (>60% lettere maiuscole)
    let ui_noise = [
        "fehu", "dashboard", "transazioni", "categorie", "dati", "foto", "obiettivi",
        "uscita", "entrata", "importo", "descrizione", "categoria", "metodo",
        "contanti", "carta", "pagamento", "shopping", "servizi", "altro",
    ];
    for line in raw_text.lines() {
        let trimmed = line.trim();
        if trimmed.len() < 3 { continue; }
        let low = trimmed.to_lowercase();
        if ui_noise.iter().any(|w| low.contains(w)) { continue; }
        let letters: usize = trimmed.chars().filter(|c| c.is_alphabetic()).count();
        let uppers: usize = trimmed.chars().filter(|c| c.is_uppercase()).count();
        if letters > 2 && uppers * 100 / letters.max(1) >= 60 {
            let merchant_tokens: Vec<&str> = trimmed
                .split_whitespace()
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

    // Step 2: Ollama LLM → categoria (con fallback keyword se Ollama non disponibile)
    let merchant = data.descrizione.as_deref().unwrap_or("unknown transaction");
    let parse_prompt = format!(
        "Reply with ONE word only from this list: Cibo, Trasporti, Casa, Salute, Svago, Abbigliamento, Istruzione, Sport, Lavoro, Altro.\n\
Cibo=food/restaurants; Trasporti=transport/fuel/flights; Casa=utilities/rent/home; Salute=pharmacy/medical; \
Svago=streaming/cinema/games; Abbigliamento=clothing; Istruzione=education/books; Sport=gym/sports; \
Lavoro=hosting/cloud/software/domains/Cloudflare/AWS/Adobe/professional; Altro=everything else.\n\
Transaction: {merchant}\nCategory:"
    );
    let parse_body = serde_json::json!({
        "model": ollama_model,
        "prompt": parse_prompt,
        "stream": false,
        "keep_alive": 0
    });

    let valid_categories = [
        ("cibo", "Cibo"), ("trasporti", "Trasporti"), ("casa", "Casa"),
        ("salute", "Salute"), ("svago", "Svago"), ("abbigliamento", "Abbigliamento"),
        ("istruzione", "Istruzione"), ("sport", "Sport"), ("lavoro", "Lavoro"), ("altro", "Altro"),
    ];

    let ollama_endpoint = format!("{}/api/generate", ollama_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let ollama_result = client
        .post(&ollama_endpoint)
        .json(&parse_body)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await;

    match ollama_result {
        Ok(resp) => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                let word = json["response"]
                    .as_str()
                    .unwrap_or("")
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_matches(|c: char| !c.is_alphabetic())
                    .to_lowercase();
                for (key, display) in &valid_categories {
                    if word == *key {
                        data.categoria = Some(display.to_string());
                        data.categoria_source = Some("ollama".into());
                        break;
                    }
                }
            }
        }
        Err(_) => {}
    }

    // Keyword fallback se Ollama non ha restituito una categoria
    if data.categoria.is_none() {
        if let Some(cat) = categorize_by_keywords(&raw_text) {
            data.categoria = Some(cat);
            data.categoria_source = Some("keyword".into());
        }
    }

    Ok(data)
}
