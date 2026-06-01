# QA Checklist — Fehu

Da eseguire prima di ogni release su macchina pulita (o dopo reset dati).

---

## Installazione

- [ ] DMG si monta e app si trascina in `/Applications`
- [ ] App si apre senza blocco Gatekeeper (o `xattr -dr` funziona)
- [ ] Primo avvio mostra onboarding
- [ ] Tema dark/light si persiste al riavvio

---

## Sidebar e navigazione

- [ ] Tutte le voci sidebar navigano alla pagina corretta
- [ ] Voce attiva evidenziata
- [ ] Toggle tema dark/light funziona e cambia tutti i colori
- [ ] Sidebar visivamente corretta in entrambi i temi

---

## Dashboard

- [ ] KPI cards mostrano valori corretti (entrate / uscite / saldo)
- [ ] Saldo contanti e carta corretti
- [ ] Patrimonio totale = contanti + carta
- [ ] Grafico mensile si popola
- [ ] Donut categorie si popola
- [ ] Sankey flusso si popola
- [ ] Rettifica saldo: crea, mostra in lista, elimina
- [ ] Budget alert appare se categoria supera limite
- [ ] Bottone Aggiorna ricarica i dati

---

## Transazioni

- [ ] Lista si carica con paginazione / scroll
- [ ] Filtro per mese funziona
- [ ] Filtro per categoria funziona
- [ ] Ricerca testo funziona
- [ ] Vista lista e vista calendario si alternano
- [ ] Calendario mostra dot verde/rosso sulle date
- [ ] Nuova transazione: tutte le tipologie (entrata / uscita / contanti / carta)
- [ ] Modifica transazione esistente
- [ ] Elimina transazione
- [ ] Creazione categoria inline funziona
- [ ] Allegato: upload file funziona
- [ ] Allegato: visualizza / scarica / elimina
- [ ] Badge contanti (C) e carta (K) visibili e corretti
- [ ] Cmd+N apre nuova transazione
- [ ] Cmd+F porta il focus sulla ricerca

---

## Categorie

- [ ] Lista categorie si carica
- [ ] Nuova categoria: nome + icona + colore
- [ ] Icona picker mostra tutte le icone
- [ ] Color picker funziona
- [ ] Budget mensile opzionale si salva
- [ ] Modifica categoria esistente
- [ ] Elimina categoria (warning se ha transazioni)

---

## Obiettivi

- [ ] Lista obiettivi si carica
- [ ] Nuovo obiettivo: nome + target + data
- [ ] Barra progresso si aggiorna
- [ ] Versamento: aggiunge importo e crea transazione uscita
- [ ] Elimina obiettivo

---

## Ricorrenti

- [ ] Lista template si carica
- [ ] Nuovo template: tutte le frequenze (giornaliera / settimanale / mensile / annuale)
- [ ] Template si attiva alla data prevista (verificare con data test)
- [ ] Modifica template
- [ ] Elimina template

---

## P.IVA

- [ ] Regime Forfettario calcola correttamente
- [ ] Regime Ordinario calcola correttamente
- [ ] Regime Semplificato calcola correttamente
- [ ] Varianti INPS funzionano
- [ ] Scenari −20% / base / +20% mostrano valori diversi

---

## Foto / OCR

- [ ] Drag-drop immagine ricevuta funziona
- [ ] Tesseract estrae importo / data / commerciante
- [ ] Categorizzazione automatica (con Ollama attivo)
- [ ] Fallback keyword senza Ollama
- [ ] Transazione pre-compilata modificabile e salvabile

---

## Dati (Export)

- [ ] Export CSV con filtro date funziona e si scarica
- [ ] Export XLSX funziona e si scarica
- [ ] Import XLSX "Registro Lavoro" importa le righe
- [ ] Backup database: scarica file `.db`
- [ ] Restore database: ripristina da file `.db`

---

## Impostazioni

- [ ] Salva URL Ollama
- [ ] Salva percorso Tesseract (o auto-rilevamento)
- [ ] Salva simbolo valuta
- [ ] Token Telegram si salva
- [ ] Bot Telegram: avvia / ferma
- [ ] Controlla aggiornamenti: risponde correttamente

---

## Bot Telegram

- [ ] Bot risponde a `/start`
- [ ] Invio foto ricevuta funziona
- [ ] Aggiunta transazione via bot
- [ ] Notifica sul desktop quando bot riceve messaggio

---

## Aggiornamenti

- [ ] Check aggiornamenti trova nuova versione (testare con versione vecchia)
- [ ] Popup aggiornamento appare
- [ ] Download e install funziona
- [ ] Badge su icona Impostazioni appare se update disponibile

---

## Tema chiaro — verifica visiva

- [ ] Sidebar colori corretti
- [ ] Card KPI leggibili
- [ ] Chart.js colori adattati
- [ ] Badge cash/card leggibili
- [ ] Bottoni bot start/stop leggibili
- [ ] Nessun testo illeggibile (testo chiaro su sfondo chiaro)
- [ ] Nessun elemento nero/scuro hardcoded che stride

---

## Keyboard shortcuts

- [ ] `Cmd+N` → nuova transazione
- [ ] `Cmd+F` → focus ricerca
- [ ] `Cmd+,` → impostazioni
