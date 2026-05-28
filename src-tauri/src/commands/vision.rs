use crate::{error::{AppError, AppResult}, models::ReceiptData};
use base64::{engine::general_purpose::STANDARD, Engine};

#[tauri::command]
pub async fn analyze_receipt(image_path: String) -> AppResult<ReceiptData> {
    let bytes = std::fs::read(&image_path)
        .map_err(|e| AppError::Validation(format!("Impossibile leggere il file: {e}")))?;
    let b64 = STANDARD.encode(&bytes);

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "moondream",
        "prompt": "Analizza questo scontrino italiano. Estrai le informazioni e rispondi SOLO con un oggetto JSON in questo formato esatto (senza testo aggiuntivo):\n{\"importo\": 12.50, \"data\": \"2024-01-15\", \"descrizione\": \"nome negozio o prodotto principale\", \"categoria\": \"Cibo\"}\nCategorie disponibili: Cibo, Trasporti, Casa, Salute, Svago, Abbigliamento, Istruzione, Sport, Lavoro, Altro\nSe non riesci a leggere un campo usa null.",
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

    // Extract JSON substring from response (model may add extra text)
    let start = text.find('{').ok_or_else(|| AppError::Validation("Nessun JSON nella risposta".into()))?;
    let end = text.rfind('}').ok_or_else(|| AppError::Validation("JSON malformato".into()))? + 1;
    let json_str = &text[start..end];

    let data: ReceiptData = serde_json::from_str(json_str)
        .map_err(|e| AppError::Validation(format!("Impossibile parsare la risposta: {e}. Risposta raw: {json_str}")))?;

    Ok(data)
}
