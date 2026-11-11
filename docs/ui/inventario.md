# Inventario del mock della UI

Fonte per la specifica della UI (`docs/SPEC-UI.md`, DEC-027). Descrive il mock in `project/` così com'è, con i valori
esatti presi dal CSS e dagli script, e segna con **[CORREZIONE]** i punti che `NOTE.md`, la SPEC o le decisioni
cambiano. Le proposte che non vengono da un documento sono marcate *(proposta)*; i valori non determinabili dal mock
lo dicono esplicitamente.

Tavole lette per intero: `Portolano App.dc.html` (la più aggiornata, prevale in caso di conflitto),
`Portolano Design System.dc.html` (DS), `Portolano Editor.dc.html`, `Portolano Dialoghi.dc.html`,
`Portolano Brand.dc.html`. `support.js` è il runtime del prototipo: traduce gli attributi `style-hover`,
`style-active`, `style-focus` in `:hover`, `:active`, `:focus` e non contiene animazioni dell'interfaccia (solo lo
scintillio dei segnaposto del prototipo, da ignorare).

Convenzioni: i nomi dei token proposti seguono il prefisso `--pl-` della DS; fra parentesi il nome breve usato nella
tavola App (`--tx2`, `--ac`…). "Vetro" = superficie semitrasparente con `backdrop-filter`.

---

## 1. Brand applicato

### 1.1 Direzione scelta
**"Arcipelago · Grafite" (stile 2a)**, dalla tavola Brand "Turno 2 · Revisione di 1b — Arcipelago, senza neon".
Tutte le tavole operative la dichiarano ("Design system · stile 2a Grafite · tema scuro", "Schermate 3 e 4 · stile 2a
Grafite", …) e nella App la palette `2a` è quella predefinita (`palette` default `"2a"`).
Descrizione del brand: "Grafite caldo e verde bottiglia. Nessun bagliore: l'enfasi è un riempimento pieno."
Tre regole del sistema (DS): l'enfasi è un riempimento pieno e mai un bagliore (`--glow: none`), la pillola è riservata
ai badge, il terminale è sempre opaco.

Direzioni scartate (solo presentazione, da non implementare): Turno 1 "1a Lossodromia" (Geist, ciano), "1b Arcipelago"
(neon smeraldo, origine della 2a), "1c Astrolabio" (Schibsted Grotesk, ambra); Turno 2 "2b Arcipelago · Sabbia".
La App le espone ancora come Tweak (`2a`, `2b`, `1b`): vedi §8.

### 1.2 Logo
Marchio = una rotta che disegna una **"P"** passando per dei "porti" (vertici). `viewBox="0 0 64 64"`, `fill="none"`:
- tracciato: `<polyline points="20,54 20,12 40,12 48,22 40,32 20,32">` (asta verticale dal basso, poi l'occhiello),
  `stroke-linejoin="round"`;
- vertici: cerchi sui punti della rotta.

Esistono **due varianti** (ambiguità §9):

| | Tavola Brand 2a (96 px) | Tavola App: toolbar (24 px) e primo avvio (30 px) |
|---|---|---|
| Tratto | `stroke="#7FC2A0"`, `stroke-width="2.4"` | `stroke: var(--ac-tx)` (#7FC2A0), `stroke-width="4"` |
| Vertici | 6 cerchi `r="4.5"`: origine (20,54) piena `#7FC2A0`; (20,12) (40,12) (48,22) (40,32) (20,32) vuoti, `fill="#141311"` (fondo), `stroke="#C9C3B6"`, `stroke-width="2.2"` | 4 cerchi `r="5"` pieni `var(--ac-tx)` in (20,54) (20,12) (48,22) (20,32); nessun cerchio in (40,12) e (40,32) |
| Uso | marchio grande | marchio piccolo, monocromo |

La variante App è quella da usare nell'interfaccia (a 24–30 px i cerchi vuoti col tratto 2.2 non si leggono). L'icona
dell'applicazione (icns/png) **non è disegnata**: la tavola Brand dice "I marchi sono schizzi geometrici, da rifinire
dopo la scelta".

**Logotipo**: "portolano" tutto minuscolo, Hanken Grotesk.
- Toolbar: 16 px, peso 600, `letter-spacing: -.02em`.
- Primo avvio: 22 px, peso 600, `-.02em`, accanto al marchio da 30 px con `gap: 10px`.
- Tavola Brand: 60 px, peso 500, `-.03em`, `line-height: 1`; motto "tutti i tuoi porti, a un clic" in IBM Plex Mono
  14 px `#A8A297` (il motto non compare nella App).

### 1.3 Palette del brand (tavola Brand 2a) e differenze con la App
| Nome Brand | Brand 2a | App (vince) |
|---|---|---|
| Fondo | `#141311` | `#1B1A17` |
| Superficie | `#1E1C19` | vetro `rgba(58,54,48,.16)` |
| Bordo | `#35322D` | `rgba(255,245,225,.16)` |
| Testo | `#EDE9E1` | `#EDE9E1` |
| Testo 2 | `#A8A297` | `#A8A297` |
| Bottiglia | `#2F7D5B` | `#2F7D5B` (accento) |
| Bottiglia chiaro | `#7FC2A0` | `#7FC2A0` (accento testo) |
| Successo / Avviso / Errore / Info | `#9BD06E` / `#E0A84B` / `#E5695F` / `#7FB2D9` | uguali |
| Nodi SSH / RDP / Credenziale / Cartella | `#7FC2A0` / `#7FA6D9` / `#D9B45F` / `#A8A297` | uguali |
| Non supportato | `#6B665D` | `#8A8479` |

La tavola Brand descrive anche un vetro "al 64%, blur 30 px" e un badge "Password assente" pieno (`#3A2E1A` su
`#E0A84B`): la App li ha sostituiti (vetro 16% blur 6, badge tratteggiato). Valgono i valori della App.

---

## 2. Design token

Valori della palette 2a (tavola App, `THEMES['2a']` e `body{…}`), confrontati con l'esportazione `:root` della DS.

### 2.1 Colori — superfici
| Token proposto (App) | Valore | Uso |
|---|---|---|
| `--pl-bg` (`--bg`) | `#1B1A17` | fondo della finestra (dietro il canvas); anello interno del campione colore selezionato |
| `--pl-glass` (`--glass`) | `rgba(58,54,48,.16)` | pannelli: toolbar, barra laterale, barra delle tab, dettaglio, validazione, testata di sessione, riquadri RDP/errore, stato vuoto |
| `--pl-glass-modal` (`--modal`) | `rgba(40,37,33,.72)` | dialoghi e pannello dell'editor |
| `--pl-popover` *(proposta, oggi letterale)* | `rgba(40,37,33,.92)` | menu contestuale, select aperta, toast (senza blur) |
| `--pl-terminal` (`--term`) | `#0C0B0A` | terminale, anteprima comando, XML grezzo del nodo non supportato, riga di errore della sessione, tooltip |
| `--pl-field` (`--field`) | `rgba(255,255,255,.07)` | fondo di input, select, segmented, bottoni secondari dei dialoghi, filtri non attivi, tessere icona |
| `--pl-border` (`--bd`) | `rgba(255,245,225,.16)` | bordi di pannelli, campi, righe, separatori, linee della rotta non accese, colore della scrollbar |
| `--pl-hover` (`--hover`) | `#2A2723` | hover delle righe dell'albero; fondo della riga trascinata |
| `--pl-active` (`--active`) | `#24382E` | selezione (riga dell'albero, tab, modalità credenziale), hover dei bottoni, chip di modalità, step già fatti |
| `--pl-overlay-panel` *(proposta)* | `rgba(0,0,0,.4)` | velo dietro l'editor laterale |
| `--pl-overlay-dialog` *(proposta)* | `rgba(0,0,0,.45)` | velo dietro i dialoghi (la tavola Dialoghi dice "si scurisce al 40%": ambiguità §9) |
| `--pl-edge` | `inset 0 1px 0 rgba(255,255,255,.08)` | filo di luce in alto su pannelli, dialoghi, menu |
| `--pl-terminal-border` *(proposta)* | `rgba(255,255,255,.08)` | bordo 1 px di terminale e blocchi `--term` |

Tinte bianche usate come letterali (da trasformare in token *(proposta)*):

| Valore | Uso |
|---|---|
| `rgba(255,255,255,.04)` | riquadro "Credenziale effettiva", riquadro "Da dove arriverà", riquadro nodo non supportato |
| `rgba(255,255,255,.05)` | hover delle righe di validazione; fondo dei chip della rotta nel dettaglio |
| `rgba(255,255,255,.07)` | = `--field`: bottoni della toolbar (+ Connessione…), campo di ricerca, filtri, tessera icona del dettaglio |
| `rgba(255,255,255,.08)` | bottoni secondari del dettaglio e dello stato vuoto, tag nel dettaglio, hover di "Elimina" |
| `rgba(255,255,255,.1)` | tag nell'editor, chip "dall'albero"/"indirizzo libero", bottone "Scegli…" del selettore, binario delle barre "Per tipo" |
| `rgba(255,255,255,.12)` | binario della barra di avanzamento delle password d'import |
| `rgba(255,255,255,.16)` | binario del toggle spento; "semafori" finti della finestra |
| `rgba(255,255,255,.25)` | segmento "Saltati" della barra del report |
| `rgba(255,255,255,.35)` | pallino "Saltati" nella tessera del report |

### 2.2 Colori — testo
| Token DS | Nome App | Valore | Uso |
|---|---|---|---|
| `--pl-text` | `--tx` | `#EDE9E1` | testo principale, titoli, valori |
| `--pl-text-2` | **`--tx3`** | `#CFC9BD` | testo secondario leggibile: descrizioni, valori del dettaglio, messaggi, bottoni testuali, pomello del toggle spento |
| `--pl-text-3` | **`--tx2`** | `#A8A297` | etichette di campo, meta, didascalie, placeholder, chevron, opzione disattiva |

**Attenzione**: i nomi sono invertiti fra App e DS (App `--tx2` = muted `#A8A297`, DS `--pl-text-2` = `#CFC9BD`). La
specifica deve adottare un solo schema *(proposta: `--pl-text`, `--pl-text-secondary` #CFC9BD, `--pl-text-muted`
#A8A297)*. Nel resto di questo documento `tx2` = `#A8A297`, `tx3` = `#CFC9BD`, come nella App.

### 2.3 Colori — accento
| Token | Valore | Uso |
|---|---|---|
| `--pl-accent` (`--ac`) | `#2F7D5B` | bottone primario, opzione selezionata (segmented, select), filtro attivo, toggle acceso, badge "Nel portachiavi", step corrente, chip di destinazione delle catene, voce evidenziata del menu |
| `--pl-accent-hover` (`--ac-h`) | `#38916A` | hover del primario |
| `--pl-accent-text` (`--ac-tx`) | `#7FC2A0` | link, campo a fuoco, logo, "Vai al nodo →", evidenza della ricerca, prompt e cursore del terminale, linee dello stepper completate, barra di avanzamento, stato "in corso", segmento "Importati" |
| `--pl-on-accent` (`--on-ac`) | `#F5F1E8` | testo su accento; pomello del toggle acceso |
| `--pl-on-error` *(proposta)* | `#1B0F0D` | testo del bottone distruttivo |
| `--pl-error-hover` *(proposta)* | `#EE8279` | hover del bottone distruttivo (solo DS) |
| `--glow` | `none` | ombra di hover del primario (in 2a non c'è bagliore) |

Link (`a`): colore `--ac-tx`, hover `--tx`.

### 2.4 Colori — semantici
| Token | Valore | Uso |
|---|---|---|
| `--pl-ok` (`--ok`) | `#9BD06E` | connesso, lucchetto nel piè della barra laterale, password presente (meta e testo), "Leggibile", "Le due password coincidono" e bordo della conferma |
| `--pl-warn` (`--warn`) | `#E0A84B` | password assente (badge tratteggiato), Avviso, Parziale, riquadro "È usata da N nodi", pallino della tab Validazione |
| `--pl-error` (`--err`) | `#E5695F` | Errore, connessione fallita, "Elimina", bottone distruttivo, input in errore |
| `--pl-info` (`--info`) | `#7FB2D9` | Nota (livello Info della validazione) |

### 2.5 Colori — tipi di nodo e protocolli
| Token | Valore | Uso |
|---|---|---|
| `--pl-node-ssh` (`--ssh`) | `#7FC2A0` | icona SSH, barra "SSH" del report, pallino della destinazione SSH nella rotta |
| `--pl-node-rdp` (`--rdp`) | `#7FA6D9` | icona RDP, barra "RDP" |
| `--pl-node-cred` (`--cred`) | `#D9B45F` | icona credenziale, stella dei preferiti, barra "Credenziali" |
| `--pl-node-folder` (`--folder`) | `#A8A297` | icona cartella, barra "Cartelle" |
| `--pl-node-unsupported` (`--uns`) | `#8A8479` | icona non supportato, segmento e tessera "Non supportati", barra "Altri tipi" |

Colori per il campo "Colore" del nodo (editor, passo 1), cerchi da 20 px: `#7FC2A0`, `#7FA6D9`, `#D9B45F`,
`#D98F7F`, `#B79FD9`, `#A8A297`. I due non presenti altrove (`#D98F7F` terracotta, `#B79FD9` lilla) vanno aggiunti
come token *(proposta: `--pl-swatch-1..6`)*.

### 2.6 Colori del canvas di sfondo e varie
| Valore | Uso |
|---|---|
| `201,195,182` (= `#C9C3B6`) | linee della costellazione (`lc` di 2a) |
| `127,194,160` (= `#7FC2A0`) | punti e anelli della costellazione (`dc` di 2a) |
| `#F5F1E8` / `#CFC9BD` | pomello del toggle acceso / spento (letterali) |

### 2.7 Tipografia
**Famiglie**
| Token | Valore | Note |
|---|---|---|
| `--pl-font-ui` | `'Hanken Grotesk', system-ui, sans-serif` | tutta l'interfaccia. La App scrive `'Hanken Grotesk',sans-serif` (senza `system-ui`): usare la versione DS |
| `--pl-font-mono` | `'IBM Plex Mono', ui-monospace, monospace` | host, porte, utenti, percorsi, comandi, terminale, intestazioni di gruppo, numeri tecnici |

**Provenienza**: entrambe da Google Fonts,
`https://fonts.googleapis.com/css2?family=Hanken+Grotesk:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500&display=swap`.
Nell'app vanno **incluse localmente** (file woff2 nel bundle, `@font-face` con `font-display: swap`), niente rete.
Pesi da includere: Hanken Grotesk 400, 500, 600 (il 700 è caricato dal mock ma **mai usato**); IBM Plex Mono 400, 500.
Entrambe sono sotto SIL Open Font License. `font-variant-numeric: tabular-nums` è usato su numeri del report e sulle
barre "Per tipo": verificare che il file di Hanken Grotesk incluso abbia la feature `tnum` (non determinabile dal mock).

**Pesi**: 400 (testo), 500 (nomi nelle righe, bottoni secondari, mono delle intestazioni), 600 (titoli, primario,
opzione selezionata, etichette di gravità).

**Scala** (px · peso · interlinea · spaziatura · uso)
| Ruolo | Dimensione | Peso | Altro | Uso |
|---|---|---|---|---|
| Numero hero | 64 | 600 | `line-height:1`, `-.04em`, tabular | totale del report d'import |
| Numero tessera | 34 | 600 | `-.03em`, tabular | tessere del report |
| Titolo del nodo | 26 | 600 | `-.02em` | nome nel dettaglio (DS: "Hanken Grotesk 26 / 600") |
| Titolo stato vuoto | 24 | 600 | `-.02em` | "Il documento è vuoto" |
| Logotipo grande | 22 | 600 | `-.02em` | primo avvio |
| Contatore mono | 22 | 500 mono | — | "3 / 12" nell'import |
| Titolo di pannello | 20 | 600 | `-.01em` | titolo dell'editor, "4 problemi nel documento", titoli dei riquadri RDP/errore |
| Titolo di dialogo | 18 | 600 | — | tutti i dialoghi, titoli nel riquadro "Da dove arriverà" |
| Logotipo / titolo area di rilascio | 16 | 600 | `-.02em` (logo) | toolbar; "Trascina qui…" |
| Titolo di sezione | 15 | 600 | — | schede del primo avvio (DS: "Titolo di sezione 15 / 600") |
| Testo in evidenza | 14.5 | 400 | `line-height:1.55` | testo dello stato vuoto, testo "Autenticazione come…" |
| Mono grande | 14 | 400/500 | — | riepilogo `user@host:port` del dettaglio e della sessione, valori Utente/Dominio |
| **Testo UI (base)** | **13.5** | 400/500 | — | `font-size` della radice; compatto **12.5** (DS) |
| Bottoni | 13 | 500 (600 primario) | — | tutti i bottoni (13.5 nello stato vuoto) |
| Testo dei dialoghi / mono comando | 13 | 400 | mono `line-height:1.6` | testi di spiegazione nei dialoghi, comando, terminale (`13px/1.65`) |
| Secondario | 12.5 | 400 | — | chip, didascalie, motivi del report, path mono nella validazione |
| Etichetta di campo | 12 | 400 | colore `tx2` | etichette sopra i campi, filtri, didascalia del comando, conteggi |
| Intestazione di gruppo | 11.5 | 500 mono | `letter-spacing:.1em`, `text-transform:uppercase`, `tx2` | "ROTTA", "CREDENZIALE EFFETTIVA", "GENERALE", "JUMP HOST · …", "DA DOVE ARRIVERÀ"… |
| Meta dell'albero | 11.5 | 400 mono | — | host/stato a destra nelle righe |
| Piccolo | 11 | 400 | — | tipo nella tab ("SSH"), stella nell'albero |
| Chevron | 10 | — | — | ▾/▸ |

**Interlinee**: `1` (numero hero), `1.45` (messaggi di validazione, tooltip multilinea), `1.5` (testo corrente,
spiegazioni), `1.55` (stato vuoto), `1.6` (comando, errore mono), `1.65` (terminale, XML grezzo). Default del browser
altrove (non determinato; *proposta* `1.4`).

**Spaziature delle lettere**: `-.04em`, `-.03em`, `-.02em`, `-.01em` (titoli, vedi sopra), `.1em` (intestazioni di
gruppo), `.2em` (pallini delle password), `-2px` (glifo della maniglia "⋮⋮").

### 2.8 Spaziature
Scala dichiarata dalla DS: `--pl-s1: 4px` · `--pl-s2: 8px` · `--pl-s3: 12px` · `--pl-s4: 16px` · `--pl-s5: 20px` ·
`--pl-s6: 24px` · `--pl-s7: 32px` · `--pl-s8: 48px`.
Il mock però usa anche valori fuori scala, da riprodurre esattamente: `2, 3, 5, 6, 7, 9, 10, 11, 13, 14, 18, 22, 26,
28, 30, 36 px`. I più frequenti:
- gap: `5px` (etichetta→campo), `6px`, `7px`, `8px`, `10px`, `12px` (griglia principale), `14px`, `16px`, `20px`, `24px`;
- padding: finestra `14px`; toolbar `10px 14px`; pannello dettaglio `26px 28px`; editor/validazione `24px`;
  dialoghi `22px` (centrati `26px 22px`; report e preferenze `26px`); campo `9px 12px`; bottone `8px 14px`/`8px 16px`;
  segmento `6px 12px` (o `6px 14px`); riga lista/campo composto `8px 10px`; riquadri interni `12px 14px`, `14px 16px`,
  `18px 20px`.

### 2.9 Raggi
| Token DS | Valore | Uso |
|---|---|---|
| `--pl-r-seg` | `7px` | opzione del segmented, voce della select aperta |
| `--pl-r-row` (`--rr`) | `9px` | riga dell'albero, tab, righe del picker |
| `--pl-r-ctl` (`--rc`) | `10px` | bottoni, campi, select, segmented, tessere icona, righe della lista ordinabile, toast-riga file |
| `--pl-r-card` | `12px` | riquadri interni, comando, menu contestuale, toast, schede del primo avvio, tessere del report |
| *(non in DS)* `--pl-r-inner` *(proposta)* | `14px` | barra delle tab, testata di sessione, terminale, riquadro credenziale, tessera icona 52 px, area di rilascio, riquadro "Da dove arriverà" |
| `--pl-r-panel` | `18px` | pannelli e dialoghi |
| `--pl-r-pill` | `999px` | badge, chip, filtri, toggle, step, barra del report |
| — | `8px` | tooltip, voci del menu contestuale |
| — | `6px` | barre "Per tipo" |
| — | `4px` | barra di avanzamento d'import, "×" delle tab |
| — | `2px` / `3px` | linee di collegamento (2 px), linee della rotta di sessione (3 px) |
| — | `50%` | pallini, punti, cerchi colore |

La DS elenca sei raggi e non il 14 px, che la App usa in 16 punti: va aggiunto (ambiguità §9).

### 2.10 Ombre
| Token | Valore | Uso |
|---|---|---|
| `--pl-edge` | `inset 0 1px 0 rgba(255,255,255,.08)` | pannelli, dialoghi, menu |
| `--pl-shadow-modal` | `0 24px 60px rgba(0,0,0,.4)` | dialoghi (insieme a `--pl-edge`), menu contestuale |
| `--pl-shadow-drag` | `0 8px 24px rgba(0,0,0,.35)` | riga trascinata, select aperta, toast, tooltip |
| campione selezionato | `0 0 0 2px var(--bg), 0 0 0 4px <colore>` | anello del colore scelto |
| bersaglio del drop | `inset 0 2px 0 var(--ac-tx)` | linea d'inserimento sopra la riga dell'albero |

L'editor laterale ha solo `--pl-edge` (niente `--pl-shadow-modal`). Barra delle tab e testata di sessione non hanno
`--pl-edge`.

### 2.11 Vetro e blur
| Livello | Fondo | `backdrop-filter` | Bordo | Uso |
|---|---|---|---|---|
| Pannello | `rgba(58,54,48,.16)` | `blur(6px) saturate(140%)` (`--pl-blur-panel`) | `1px solid --bd` | toolbar, laterale, tab, dettaglio, sessione |
| Dialogo | `rgba(40,37,33,.72)` | `blur(18px)` (`--pl-blur-modal`) | `1px solid --bd` | dialoghi, editor |
| Popover | `rgba(40,37,33,.92)` | nessuno | `1px solid --bd` | menu, select, toast |
| Terminale | `#0C0B0A` | nessuno ("opaco · senza blur") | `1px solid rgba(255,255,255,.08)` | terminale, comando, XML |

Includere sempre anche `-webkit-backdrop-filter` (WebKit di macOS in Tauri).

### 2.12 Opacità
- `.45`: bottone "Connetti" della toolbar quando la selezione non è una connessione (o albero vuoto); riga
  dell'albero mentre è trascinata.
- Canvas: alfa calcolate a runtime (§3).

### 2.13 z-index
**Non presenti nel mock**: l'ordine è dato dal DOM (canvas assoluto → griglia relativa → velo dell'editor → velo dei
dialoghi). Un dialogo aperto dall'editor sta sopra l'editor. *(Proposta)*: `--pl-z-canvas: 0`, `--pl-z-app: 1`,
`--pl-z-panel: 10` (editor), `--pl-z-dialog: 20`, `--pl-z-popover: 30` (menu, select, tooltip), `--pl-z-toast: 40`.

### 2.14 Movimento
| Token | Valore | Uso dichiarato |
|---|---|---|
| `--pl-t-fast` | `160ms` | hover, fuoco, colori, apertura popover, menu, dialoghi, tooltip |
| `--pl-t-base` | `260ms` | pannelli (editor), segmented, cambio di stato dei badge, pallini di stato, passo del wizard, toast |
| `--pl-t-slow` | `480ms` | rotta di connessione, contatori del report, lampeggio di righe |
| `--pl-ease-spring` | `cubic-bezier(.34,1.56,.64,1)` | "per ciò che entra o si sposta" (editor, toast, righe che scorrono, rotta, bottone primario) |
| `--pl-ease-out` | `cubic-bezier(.2,.8,.2,1)` | "per dissolvenze e colori" |

Altri tempi: tooltip dopo `500ms`; toast `4s` (`8s` con azione, errori fino alla chiusura); scarto `80ms` fra i pannelli
al primo avvio; scarto `60ms` fra le tessere del report; `120ms` per anello della catena "Eredita"; scuotimento `260ms`
(3 oscillazioni); calma dello sfondo `10s`; `750ms` per salto (solo prototipo, vedi §3.3).
`@media (prefers-reduced-motion: reduce) { :root { --pl-t-fast: 0ms; --pl-t-base: 0ms; --pl-t-slow: 0ms } }` (DS).

### 2.15 Densità
`[data-density="compact"] { --pl-text-size: 12.5px; --pl-row-h: 26px }`; comoda: `13.5px` / `34px`. La App applica
solo l'altezza (26 px) e lascia il testo a 13.5; la DS cambia anche il testo ("Compatto" a 12.5 px): seguire la DS.

### 2.16 Finestra e dimensioni
- Radice: `height:100vh`, `min-height:760px`, `overflow:hidden`, `color-scheme:dark`, `font-size:13.5px`,
  scrollbar `thin` color `var(--bd) transparent`.
- Griglia: `padding:14px`, righe `auto minmax(0,1fr)`, `gap:12px`; corpo a due colonne
  `minmax(220px,320px) minmax(0,1fr)`, `gap:12px`.
- **Nessun breakpoint né media query** di layout: la toolbar va a capo (`flex-wrap`), i dialoghi usano
  `width:min(<L>px, calc(100vw - 70px))`, l'editor `width:min(620px,100%)`.
- Larghezza minima della finestra: **non determinabile** (nessun vincolo orizzontale). *(Proposta)*: `minWidth` Tauri
  ≈ 900 px, `minHeight` 760 px.

---

## 3. Sfondo animato e motion

### 3.1 Costellazione (sfondo della finestra)
Tecnica: `<canvas>` 2D a pieno schermo (`position:absolute; inset:0`) dietro la griglia, ridisegnato con
`requestAnimationFrame`, limitato a **~30 fps** (salta il frame se sono passati meno di 33 ms; `dt` limitato a 0,1 s).
Nessuna animazione CSS, nessun keyframe.

Contenuto (2a):
- **120 punti** con posizione normalizzata casuale (0–1), velocità `(rand − .5) × 26` px/s per asse (±13 px/s),
  avvolti ai bordi (escono da un lato e rientrano dall'opposto). Il 12% è "grande".
- **Linee** fra coppie di punti a meno di 170 px (`d² < 28900`): spessore 1 px, colore
  `rgba(201,195,182, (1 − d²/28900) × .34 × al)`.
- **Punti**: raggio 1.7 px (grandi 3 px), `rgba(127,194,160, .75 × al)`.
- **Anello pulsante** sui punti grandi: raggio `7 + 5 × (.5 + .5 × sin(T × 1.6 + i))` px, tratto 1 px,
  `rgba(127,194,160, .3 × al)`.

Parametri (`intensita` = 8 predefinito, `theme.k` = 1.25 per 2a):
- `calm` tende a **0.25** quando "si lavora" (tab attiva ≠ dettaglio **e** ultima attività < 10 s; l'attività è la
  connessione riuscita e ogni tasto nel terminale), altrimenti a **1**; inseguimento esponenziale
  `calm += (target − calm) × min(1, dt × 2.5)` (costante ≈ 0,4 s).
- `focus` = 1 con la finestra a fuoco, **0.3** senza fuoco.
- velocità `k = (intensita/8) × calm × focus`; il tempo `T` avanza di `dt × k`.
- alfa `al = (.35 + .65 × calm) × 1.25`: a riposo 1.25 (linee fino a .425, punti .94), mentre si lavora ≈ .64.
- finestra nascosta (`document.hidden`): nessun disegno.
- **Riduci movimento** (preferenza **oppure** `prefers-reduced-motion: reduce`): velocità 0, un ridisegno al secondo,
  `al = .6 × 1.25 = .75` (costellazione ferma e attenuata).

Riassunto della DS: "Sfondo: pieno a riposo, al 25% mentre si lavora in una sessione, di nuovo pieno dopo 10 s di
pausa; al 30% della velocità senza fuoco, fermo se la finestra è nascosta o con riduci movimento."
Dialoghi: "la finestra dietro si scurisce al 40% e lo sfondo animato rallenta" (di quanto: **non determinato**; il
prototipo non lo implementa). Primo avvio: "Unico momento in cui lo sfondo animato è protagonista: resta a piena
intensità dietro il dialogo."

Note d'implementazione *(proposta)*: il prototipo imposta `canvas.width = clientWidth` senza `devicePixelRatio`
(sfocato su Retina): scalare per il DPR. Le tavole statiche (DS, Editor, Dialoghi) usano una costellazione fissa con
seme, densità `w×h/9000`, soglia `22000`, alfa .4/.9 e un'area vuota sotto il titolo: è solo presentazione.

### 3.2 Micro-interazioni
| Elemento | Comportamento |
|---|---|
| Hover generico | fondo tinto in 160 ms (righe albero `--hover`; bottoni `--active`; tessere icona `--active`; righe di validazione `rgba(255,255,255,.05)`) |
| Bottone primario | `transition: all 160ms spring`; hover `--ac-h` (+`--glow`, nullo); premuto `transform: scale(.96)` |
| Campo a fuoco | bordo `--ac-tx`, transizione 160 ms |
| Segmented | il riempimento verde "scivola" sull'opzione scelta in 260 ms |
| Campi dipendenti | L×A della risoluzione compaiono solo con "Fissa" (altezza e opacità, 160 ms); sezione Gateway chiusa finché l'host è vuoto; campi dipendenti delle Preferenze compaiono con l'opzione |
| Editor laterale | entra da destra in 260 ms, `--pl-ease-spring` |
| Dialoghi | scala da 96% a 100% e opacità, 160 ms; velo scuro |
| Popover/menu | scala da 96% a 100% dal punto del clic, 160 ms |
| Tooltip | compare dopo 500 ms, dissolvenza 160 ms |
| Toast | entra dal basso a destra con la molla (260 ms) |
| Badge segreto | dopo "Imposta" passaggio di colore 260 ms e il lucchetto si chiude |
| Password non coincidenti | bordo rosso + scuotimento orizzontale (3 oscillazioni, 260 ms); con riduci movimento solo il colore |
| Sblocco vault | il lucchetto si apre (260 ms) e il dialogo si dissolve |
| Primo avvio → finestra | dopo la scelta, toolbar, albero e area centrale entrano in sequenza: 80 ms di scarto, 260 ms ciascuno |
| Cambio credenziale della cartella | i nodi che la ereditano lampeggiano una volta nell'albero (fondo tinto, 480 ms) |
| Selettore credenziale | il riquadro "Da dove arriverà" cambia con dissolvenza 160 ms; in "Eredita" la catena si disegna da sinistra, un anello ogni 120 ms |
| Picker | l'albero si filtra mentre scrivi, le righe escluse si chiudono in 160 ms |
| Validazione | clic: l'albero scorre al nodo, lo seleziona e la riga lampeggia una volta (480 ms); dopo "Ricontrolla" i problemi risolti escono con dissolvenza 260 ms |
| Wizard d'import | fra i passi il contenuto scorre in orizzontale (260 ms) e lo stepper si riempie; passo 3: la scheda esce a sinistra e la nuova entra da destra (260 ms), la barra avanza |
| Report d'import | numeri da 0 al valore in 480 ms (decelerazione), barra che si riempie da sinistra nello stesso tempo, tessere con 60 ms di scarto; con riduci movimento valori già finali; hover su un segmento evidenzia la tessera |
| Preferenze | densità e riduci movimento cambiano la finestra dietro in tempo reale (altezza delle righe, 260 ms) |

### 3.3 Connessione: in corso, riuscita, fallita
Nel prototipo: la rotta "si accende un salto alla volta (750 ms per salto): linea 480 ms, punto 260 ms, curva
cubic-bezier(.34,1.56,.64,1). Riuscita: tutta la rotta passa al colore di successo. Fallita: i salti percorsi si
spengono, la destinazione diventa rossa e compare il messaggio con 'Riprova'." Pallino della tab e del nodo
nell'albero: `--ac-tx` in corso, `--ok` connesso, `--err` fallito, con transizione 260 ms.

**[CORREZIONE]** (NOTE.md): OpenSSH con `-J` non espone l'avanzamento per salto né la latenza. Si usa uno **stato
unico**: *connessione in corso* / *connesso* / *errore*. Niente "salto 1 di 2", niente "18 ms", niente accensione
progressiva temporizzata. *(Proposta)*: in corso, tutta la rotta in `--ac-tx` (linee e punti, eventualmente pulsazione
lenta disattivata da riduci movimento); connesso, tutta in `--ok` con la transizione 480 ms spring; errore, linee in
`--tx2`, destinazione `--err`.

### 3.4 Drag & drop
- **Albero**: la riga trascinata va a opacità .45; sopra il bersaglio compare una linea d'inserimento
  (`box-shadow: inset 0 2px 0 var(--ac-tx)`); rilasciando *dentro* una cartella il bersaglio è il fondo tinto della
  cartella (descritto in DS, non realizzato nel prototipo). Una cartella si sposta con tutto il contenuto.
  Errori possibili dal core da mostrare (toast d'errore): nome già usato da un fratello (`NameTaken`, DEC-022.5),
  profondità oltre 50 livelli (`TooDeep`, DEC-021), spostamento dentro se stessa.
- **Lista ordinabile (jump host)**: la riga afferrata si solleva (ombra `--pl-shadow-drag`, bordo `--ac-tx`, fondo
  `#2A2723`, `rotate(-1.2deg)`, maniglia in `--ac-tx`, 160 ms) e lascia un segnaposto alto 38 px, raggio 10,
  `border: 1.5px dashed var(--ac-tx)`; le altre scorrono in 260 ms con la molla; al rilascio atterra in 160 ms. La
  catena sotto la lista si aggiorna subito: un nuovo salto compare con una linea che si allunga in 480 ms.
- **Area di rilascio dell'import**: con un file sopra, bordo `--ac-tx` continuo (160 ms).

### 3.5 Riduci movimento
Preferenza "Riduci movimento" + `prefers-reduced-motion` (la DS lo dice: "Segue anche l'impostazione di sistema").
Effetti: sfondo fermo (§3.1), durate a 0 ms (DS) — ma la preferenza promette di "sostituire gli spostamenti con
dissolvenze": ambiguità §9. Esplicitati: niente scuotimento (solo colore), report con valori già finali.

---

## 4. Catalogo dei componenti

Stato di **focus da tastiera**: nel mock esiste solo per i campi (bordo `--ac-tx`). Per tutti gli altri componenti
interattivi *(proposta)*: `:focus-visible { outline: 2px solid var(--ac-tx); outline-offset: 2px }`. Gli stati non
disegnati sono indicati come "non disegnato".

### 4.1 `Button`
| Variante | Stile | Hover | Uso |
|---|---|---|---|
| `primary` | `font:600 13px`; `padding:8px 16px` (toolbar/editor `8px 18px`, sessione `7px 16px`); `radius:10`; `bg --ac`; `color --on-ac`; `border:0` | `bg --ac-h`; premuto `scale(.96)` | Connetti, Salva, Avanti, Fatto, Usa icona, Sblocca, Riprova… |
| `secondary` | `font:500 13px`; `padding:8px 14px`; `bg --field` (dettaglio e stato vuoto `.08`) | `bg --active` | Annulla, Modifica, Indietro, Sfoglia…, Esporta JSON |
| `toolbar` | come secondary, `padding:8px 13px`, `bg rgba(255,255,255,.07)`, `white-space:nowrap` | `bg --active` | + Connessione, + Cartella, + Credenziale |
| `ghost` (testuale) | `bg transparent`, `color --tx3` | toolbar `--active`, DS `--field` | Importa, Valida, Preferenze, "Salta tutte le restanti" |
| `add` (tratteggiato) | `padding:7px 12px` (DS `8px 14px`); `border:1px dashed --bd`; `bg transparent`; `color --tx3` | `bg --field` | + Proprietà, + Opzione, + Dall'albero…, + Indirizzo libero |
| `danger` | `font:600 13px`; `padding:8px 16px`; `bg --err`; `color #1B0F0D` | `bg #EE8279` | Elimina comunque |
| `danger-text` | `bg transparent`; `color --err` | `bg rgba(255,255,255,.08)` | Elimina (dettaglio), Cancella (segreto) |
| `small` | `padding:6px 12px`; `bg rgba(255,255,255,.1)` | non disegnato | Scegli… nel selettore |
| `large` | `font:13.5px`; `padding:10px 18px` | come la variante | stato vuoto |
| disabled | DS: `font:600 13px`, `bg --field`, `color --tx2`, `cursor:not-allowed`; toolbar: `opacity:.45` | — | ambiguità §9 |

Stati: focus (proposta sopra), loading **non disegnato** *(proposta: etichetta invariata + spinner 12 px o puntini,
bottone non cliccabile)*. Il primario dell'editor è spento finché non cambia qualcosa (Editor: "'Salva' resta spento
finché non cambia qualcosa").

### 4.2 `TextField`
- Contenitore etichetta: colonna, `gap:5px`; etichetta 12 px `--tx2`.
- Campo: `padding:9px 12px`; `radius:10`; `bg --field`; `border:1px solid --bd`; `min-height:38px`;
  `box-sizing:border-box`; testo 13.5 `--tx`.
- Variante `mono` (`--pl-font-mono`) per host, porta, utente, dominio, percorsi, comandi, opzioni.
- Variante `password`: puntini con `letter-spacing:.2em`; **nessun bottone "mostra password"** (Dialoghi).
- Variante `compact numeric`: `width:56px` (keepalive) o `52px` (risoluzione), `padding:7px 10px`.
- Variante `search` (barra laterale): `padding:9px 14px`, `border:1px solid transparent`, `bg rgba(255,255,255,.07)`,
  placeholder `--tx2`, `outline:none`.
- Stati: riposo (bordo `--bd`); **focus** bordo `--ac-tx` (160 ms); **errore** bordo `--err` + messaggio 12 px `--err`
  sotto (es. "La porta va da 1 a 65535"); **conferma ok** bordo `--ok` + messaggio `--ok` ("Le due password
  coincidono"); **vuoto/placeholder** `--tx2` (es. "Nessuno"); disabilitato non disegnato.

### 4.3 `TextArea`
Come `TextField`; "Note" con `min-height:52px` e `line-height:1.5` (Editor). Nella App il campo ha due
`min-height` in conflitto (52 e 38: vince 38): usare 52.

### 4.4 `Select`
Chiusa: come `TextField` con `display:flex; justify-content:space-between` e "▾" `--tx2` a destra.
Aperta (DS): lista `radius:10`, `bg rgba(40,37,33,.92)`, `border --bd`, `padding:4px`, ombra `--pl-shadow-drag`; voci
`padding:6px 9px`, `radius:7`; voce scelta `bg --ac`, `--on-ac`, 600. Apertura come popover (160 ms). Unico uso:
"Profondità colore" (mock: 16 bit, 24 bit, 32 bit; la SPEC ammette anche 15: §5.5).

### 4.5 `Toggle` e `ToggleRow`
- Binario `34×20`, `radius:999`; acceso `bg --ac`, pomello 16 px `#F5F1E8` a `left:16px; top:2px`; spento
  `bg rgba(255,255,255,.16)`, pomello `#CFC9BD` a `left:2px`.
- Transizione non specificata *(proposta: 160 ms ease-out sul pomello e sul colore)*.
- `ToggleRow`: etichetta (e descrizione 12.5 `--tx2` sotto) a sinistra, toggle a destra, `justify-content:space-between`,
  `gap:10px`. Nelle opzioni SSH/RDP in griglia `1fr 1fr`, `gap:10px 24px`.
- Disabilitato: non disegnato; serve per "X11 trusted" ("attivabile solo con X11 acceso") *(proposta: opacità .45)*.

### 4.6 `SegmentedControl`
Contenitore `display:flex; padding:3px; radius:10; bg --field; border:1px solid --bd` (spesso
`align-self:flex-start`); opzioni `padding:6px 12px` (o `6px 14px`; a tutta larghezza `flex:1; text-align:center;
padding:6px 8px`), `radius:7`; scelta `bg --ac; color --on-ac; font-weight:600`; opzione non disponibile `color --tx2`
("Chiaro · in arrivo"). Il riempimento scivola in 260 ms. Hover non disegnato *(proposta: `color --tx`)*.

### 4.7 `FilterChip`
Filtri dell'albero: `font:500 12px`, `padding:4px 10px`, `radius:999`, `border:0`; attivo `bg --ac`, `color --on-ac`;
non attivo `bg rgba(255,255,255,.07)`, `color --tx3`; transizione `background 160ms`. Voci: "Tutti", "SSH", "RDP",
"Credenziali", "★". Filtri del report: `padding:3px 10px`, 12 px, attivo `bg --active`, non attivo `bg --field`
("Tutti", "Parziali", "Non supportati").

### 4.8 `Tabs` (barra delle tab di lavoro)
- Barra: vetro pannello, `padding:6px`, `gap:6px`, `radius:14`, `overflow-x:auto`, scrollbar nascosta, `flex:none`.
- Tab: `padding:7px 12px`, `radius:9`, `gap:8px`, `white-space:nowrap`; pallino 7 px (transizione 260 ms); etichetta
  500; tipo 11 px `--tx2`; chiusura "×" `--tx2` (hover `--tx`), `padding:0 2px`, `radius:4`.
- Stati: selezionata `bg --active`; hover `bg --active`; non selezionata trasparente.
- Tipi: **dettaglio** (nome del nodo selezionato, tipo "dettaglio", pallino `--tx2`, non chiudibile); **Validazione**
  (tipo "N problemi", pallino `--warn`, chiudibile); **sessione** (nome del nodo, tipo "SSH"/"RDP", pallino di fase,
  chiudibile).

### 4.9 `SecretBadge` (presenza di un segreto)
Pillola `display:inline-flex; gap:6px; font-size:12.5px; padding:4px 10px; radius:999`, lucchetto 13 px
(`stroke-width:2`, `stroke:currentColor`).
| Stato | Stile | Icona | Testo |
|---|---|---|---|
| presente | `bg --ac`, `color --on-ac`, 600 | lucchetto chiuso `M5 7V5a3 3 0 0 1 6 0v2M4 7h8v6H4z` | "Nel portachiavi" |
| assente | `border:1px dashed --warn`, `color --warn` | lucchetto aperto `M5 7V5a3 3 0 0 1 5.8-1M4 7h8v6H4z` | "Assente" |
| vault bloccato | `border:1px solid --bd`, `color --tx3` | lucchetto chiuso | "Vault bloccato" |

Variante compatta in riga: `font-size:12px; padding:2px 9px; border:1px dashed --warn` "password assente" (credenziale
del gateway). Variante testo: "Nel portachiavi" in `--ok` (griglie del selettore). Nel selettore, modalità Inline, il
badge "Assente" è senza icona (incoerenza: usare il componente). Cambio di stato con transizione 260 ms.
**[CORREZIONE]**: con il vault su file bloccato si mostra "Vault bloccato", **non** "Assente" (NOTE.md). Il testo
"Nel portachiavi" va reso secondo il backend *(proposta: "Nel portachiavi" con Keychain, "Nel vault" con il file
cifrato; la CLI usa "nel vault"/"assente nel vault")*.

### 4.10 `StatusBadge`
Pillola `font-size:12.5px; padding:4px 10px; radius:999; border:1px solid <colore>; color <colore>; gap:6px` con
marcatore 7 px:
- "Connesso": `--ok`, pallino pieno;
- "In corso": `--ac-tx`, cerchio vuoto (`border:1.5px solid`);
- "Errore": `--err`, rombo (quadrato 7 px ruotato di 45°);
- "Avviso": `--warn`, senza marcatore in DS;
- "Nota": `--info`, senza marcatore in DS.

`CountBadge` (validazione): `padding:3px 10px; radius:999; border:1px solid <colore>; font-size:12px` ("1 errore",
"2 avvisi", "1 nota").

### 4.11 `SeverityMarker`
8 px: **Errore** rombo pieno `--err` (`transform:rotate(45deg)`); **Avviso** cerchio pieno `--warn`; **Nota** quadrato
vuoto `border:1.5px solid --info`. "La gravità è data da colore, forma e parola, mai dal solo colore."

### 4.12 `Tag`, `ModeChip`, `SourceChip`
- `Tag`: `font-size:12px; padding:3px 9px; radius:999; bg rgba(255,255,255,.1)` (dettaglio `.08`), `color --tx3`;
  nell'editor 12.5 px con " ×".
- `ModeChip` (modalità/origine della credenziale): `font-size:12.5px; padding:3px 10px` (dettaglio `4px 11px`);
  `radius:999; bg --active; color --tx`.
- `SourceChip` (lista jump): `padding:2px 8px; radius:999; bg rgba(255,255,255,.1); font-size:11.5px`
  ("dall'albero", "indirizzo libero").

### 4.13 `StatusDot`
Pallino di stato: 7 px (albero, tab), 9 px (testata di sessione); colori `transparent` (nessuna sessione), `--ac-tx`,
`--ok`, `--err`, `--tx2` (tab dettaglio), `--warn` (tab Validazione).

### 4.14 `Tooltip`
`padding:6px 10px; radius:8; bg #0C0B0A; border:1px solid --bd; font-size:12.5px; box-shadow:--pl-shadow-drag`;
scorciatoia in mono `--tx2` con `margin-left:6px` (es. "Connetti ⌘↩"). Multilinea: `max-width:260px; padding:8px 11px;
line-height:1.45` (es. "Ereditata dalla cartella Produzione. La password è nel portachiavi."). Ritardo 500 ms,
dissolvenza 160 ms.

### 4.15 `ContextMenu`
`width:220px; padding:5px; radius:12; bg rgba(40,37,33,.92); border --bd; box-shadow: --pl-edge, --pl-shadow-modal`.
Voci `padding:7px 10px; radius:8; display:flex; justify-content:space-between`; voce evidenziata `bg --ac; color
--on-ac; 600` (scorciatoia mono 400); scorciatoie mono `--tx2`; separatore `height:1px; bg --bd; margin:4px 6px`;
voce distruttiva `color --err`. Voci (connessione): "Connetti ⌘↩", "Apri in terminale esterno", "Mostra comando", —,
"Modifica… ⌘E", "Duplica ⌘D", "Preferito ★", —, "Elimina… ⌘⌫". Apertura: scala 96→100% dal punto del clic, 160 ms.
Il menu per cartelle/credenziali non è disegnato. "Duplica" richiede un'operazione nuova che rigeneri id e `SecretRef`
(progress.md, DEC-016.4).

### 4.16 `Dialog`
Contenitore: velo `position:absolute; inset:0; bg rgba(0,0,0,.45); padding:20px; overflow:auto`; dialogo centrato
(`margin:auto`), `width:min(<L>px, calc(100vw - 70px))`, vetro dialogo, `radius:18`, `padding:22px` (centrati
`26px 22px`, grandi `26px`), `display:flex; flex-direction:column; gap:14px` (wizard 16, report 20).
Titolo 18/600; sottotitolo mono 12.5 `--tx2` (`margin-top:3px`). Piede: `display:flex; justify-content:flex-end;
gap:8px` (Annulla secondario + azione primaria).
Larghezze: 440 (icona, picker, password, vault), 460 (eliminazione, primo avvio), 560 (wizard 1–3), 760
(preferenze), 820 (selettore credenziale), 860 (report).
Comportamento: clic sul velo = chiudi (solo il dialogo, non l'editor sotto). Esc, trappola del fuoco, ritorno del fuoco:
**non specificati** *(proposta: Esc chiude come Annulla; fuoco intrappolato; fuoco iniziale sul primo campo o sul
bottone predefinito)*. Eliminazione: il bottone predefinito (Invio) è "Annulla".

### 4.17 `SidePanel` (editor)
Velo `rgba(0,0,0,.4)`, allineato a destra; colonna `width:min(620px,100%)`, `height:100%`, `padding:14px`,
`overflow:auto`; scheda vetro dialogo, `radius:18`, `padding:24px`, `gap:16px`, `--pl-edge` senza ombra esterna.
Entrata da destra 260 ms spring. Clic sul velo chiude editor e dialoghi aperti.

### 4.18 `Toast`
`display:flex; align-items:center; gap:10px; padding:11px 14px; radius:12; bg rgba(40,37,33,.92); border:1px solid
--bd; box-shadow:--pl-shadow-drag`. Varianti: successo (pallino 8 px `--ok`, "Password salvata nel portachiavi");
errore (bordo `--err`, rombo 8 px `--err`, "db02: connessione rifiutata", azione "Riprova"); informativo con azione
("web02 eliminato", "Annulla"). Azione = link a destra (`margin-left:auto`). Posizione: in basso a destra (misure non
determinate). Durata 4 s, 8 s con azione, errori finché non chiusi: il bottone di chiusura **non è disegnato**.
"Annulla" dopo un'eliminazione richiede la cancellazione differita dei segreti (progress.md).

### 4.19 `TreeRow` (nodo dell'albero) — tutti gli stati
Struttura: `display:flex; align-items:center; gap:8px; height:34px` (compatto 26); `padding-left: 8 + profondità×18
px` (radice 8, figli 26); `padding-right:10px`; `radius:9`; `cursor:pointer; user-select:none`; transizione
`background 160ms`.
Contenuto in ordine: chevron (span 10 px, 10 px `--tx2`: "▾" aperta, "▸" chiusa, vuoto se non cartella); icona 16 px
`stroke-width:1.8` del colore del tipo; nome 500 con ellissi; stella "★" 11 px `--cred` se preferito; meta mono 11.5
a destra con ellissi; pallino di sessione 7 px.

| Stato (DS) | Resa |
|---|---|
| Cartella aperta / chiusa | chevron ▾ / ▸; meta vuota (DS mostra il numero di elementi da chiusa, es. "2"; la App no: ambiguità §9) |
| A riposo | fondo trasparente, meta host `--tx2` |
| Hover | `bg --hover` |
| Selezionato | `bg --active` (+ stella se preferito) |
| In connessione / Connesso / Errore | pallino `--ac-tx` / `--ok` / `--err` |
| Password presente / assente (credenziale) | meta "nel portachiavi" `--ok` / "password assente" `--warn` |
| Non supportato | icona `--uns`, nome `--tx3`, meta "non supportato" |
| Bersaglio del drop | `box-shadow: inset 0 2px 0 var(--ac-tx)` |
| Trascinato | `opacity:.45` |
| Compatto | altezza 26, testo 12.5 |

Meta: connessione → host; credenziale → stato della password; non supportato → "non supportato".
**[CORREZIONE]** con vault bloccato la meta della credenziale non può dire "password assente" *(proposta: "vault
bloccato" in `--tx2`)*. Focus da tastiera e navigazione con le frecce: non disegnati (§7).

Icone dei tipi (`viewBox 0 0 16 16`, `fill:none`, `stroke-linecap/linejoin:round`):
- cartella `M2 4.5h4l1.5 1.5H14v6.5H2z`; SSH `M3 5l3 3-3 3M8 11h5`; RDP `M2 3.5h12v7H2zM6 13h4`;
  credenziale `M7 8a2 2 0 1 0-4 0 2 2 0 0 0 4 0zM7 8h6M11 8v2`;
  non supportato `M3.5 3.5l9 9M8 2.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11z`.
- Spessori: 1.8 (albero, picker, liste), 1.6 (testata dettaglio 26 px, editor 22 px, tessere icona 20 px), 1.5
  (lucchetto del vault 26 px), 2 (badge 13 px). La tavola Brand dichiara "tratto 2 px su pastiglia tinta": la App usa
  1.8.

### 4.20 `SortableList` (jump host)
Riga `display:flex; align-items:center; gap:10px; padding:8px 10px; radius:10; bg --field; border:1px solid --bd`;
maniglia "⋮⋮" `--tx2`, `cursor:grab`, `letter-spacing:-2px`; indice mono 500 12 `--tx2`; per un nodo: icona SSH 15 px,
nome 500, percorso mono 12 `--tx2`; per un indirizzo: `user@host:porta` mono 13; `SourceChip` a destra; "×" `--tx2`.
Righe con `gap:6px`. Sotto: "+ Dall'albero…" (apre il picker) e "+ Indirizzo libero" (l'input **non è disegnato**).
Trascinamento: §3.4. Tastiera per riordinare: non prevista *(proposta: Alt+↑/↓ sulla riga a fuoco)*.

### 4.21 `RouteChain` (catene di chip)
- **Rotta nel dettaglio**: chip `padding:6px 12px; radius:999; border:1px solid --bd; bg rgba(255,255,255,.05);
  font:400 12.5px mono`, pallino 7 px (`--tx2` intermedi, colore del tipo per la destinazione); connettore `24×2 px`
  `--bd` `radius:2`; `gap:8px`. Etichette: "tu", gli host dei salti, l'host di destinazione.
- **Catena nell'editor SSH**: riquadro `padding:12px; radius:12; border --bd`; chip `padding:5px 11px; border --bd;
  mono 12`; connettore `18×2`; destinazione `bg --ac; color --on-ac; mono 500`. Etichette: "tu", nomi dei salti o solo
  l'host dell'indirizzo (es. "198.51.100.40"), nome della connessione.
- **Catena "Eredita"**: chip `padding:6px 12px; border --bd`; connettore `20×2`; ultimo chip `bg --ac`, 600.

### 4.22 `SessionRoute`
`display:flex; gap:8px`; per ogni tappa: linea `flex:1; height:3px; radius:3` (transizione `background 480ms
spring`), punto 11 px con `border:2px solid` e riempimento (transizione `all 260ms spring`), etichetta mono 12
`--tx3`. Colori: acceso = colore della fase, spento `--bd`; fallita: destinazione `--err`, altri `--tx2`, punti vuoti.
**[CORREZIONE]** stato unico (§3.3).

### 4.23 `CredentialField` (campo che apre il selettore)
`display:flex; align-items:center; gap:10px; padding:8px 10px; radius:10; bg --field; border:1px solid --bd`;
`ModeChip` con la modalità; testo `--tx3` con l'origine ("dalla cartella che contiene il nodo", "Produzione →
deploy", "credenziale deploy"…) o, in RDP, l'utente mono; lucchetto 13 px `--ok` se la password c'è, altrimenti badge
"password assente"; link "Cambia" 12.5 a destra. "L'avviso 'password assente' porta alla credenziale con un clic."

### 4.24 `CredentialSelector` e `CredentialPreview` ("da dove arriverà")
Griglia `270px minmax(0,1fr)`, `gap:20px`.
- **Elenco modalità**: voci `padding:9px 12px; radius:10; gap:2px`; titolo 600, sottotitolo 12 `--tx3`; scelta
  `bg --active`; hover `--active` 160 ms. Voci: "Eredita" — "Dalla cartella che contiene il nodo"; "Nessuna" — "Decide
  il client"; "Chiedi all'avvio" — "Ogni volta, senza salvare"; "Inline" — "Utente, dominio e password qui"; "Da una
  credenziale" — "Un nodo credenziale dell'albero"; "Per nome" — "Cercata per nome al momento".
- **Riquadro "Da dove arriverà"**: `padding:18px 20px; radius:14; border --bd; bg rgba(255,255,255,.04); gap:14px;
  min-height:300px`; intestazione di gruppo "Da dove arriverà"; contenuto per modalità in §5.6. Cambio con dissolvenza
  160 ms.

### 4.25 `Stepper`
`display:flex; align-items:center; gap:8px; font-size:12.5px`. Passo: pillola `padding:4px 11px; radius:999`; corrente
`bg --ac; --on-ac; 600`; fatto `bg --active` (editor: `--tx`); futuro solo testo `--tx2` (editor: trasparente);
linee `flex:1; height:2px` (`--ac-tx` se superata, `--bd` altrimenti), transizioni 260 ms. Editor: "1 Generale", "2
Connessione" (SSH/RDP) / "2 Accesso" (credenziale) / "2 Credenziale predefinita" (cartella); i passi sono cliccabili.
Wizard: "1 File", "2 Opzioni", "3 Password", "4 Report"; il passo 3 appare **barrato** se "Chiedi le password" è spento
(resa non disegnata).

### 4.26 Altri componenti
| Componente | Descrizione |
|---|---|
| `GlassPanel` | vetro pannello, `radius:18`, `--pl-edge` |
| `Card` (riquadro interno) | `radius:12` o `14`, `border:1px solid --bd`, eventuale `bg rgba(255,255,255,.04)` |
| `GroupHeader` | mono 500 11.5, `.1em`, maiuscolo, `--tx2`; nell'editor dopo il primo `margin-top:4px` |
| `FormField` | etichetta 12 `--tx2` + controllo, `gap:5px` |
| `KeyValueGrid` | griglia `150px minmax(0,1fr)` (dettaglio: `row-gap:12px; column-gap:16px`), `110px 1fr` (selettore, `row-gap:8px`), `90px 1fr` (import), `140px 1fr` (RDP, `row-gap:9px`) |
| `SecretRow` | riquadro `padding:12px 14px; radius:12; border --bd; gap:10px`: etichetta `width:110px` 12.5 `--tx2`, `SecretBadge`, azioni a destra ("Sostituisci…", "Cancella" oppure "Imposta…") |
| `KeyValueEditor` | proprietà custom: griglia `150px 86px minmax(0,1fr) 20px`, `gap:8px`; celle `padding:8px 10px; radius:10; bg --field; border --bd`; tipo "Testo"/"Segreto" in `--tx3`; segreto = badge + link "Sostituisci"; "×". Opzioni libere: griglia `1fr 1fr 20px`, celle mono |
| `TagInput` | `padding:7px 8px; radius:10; bg --field; border --bd; gap:6px`; tag "web ×"; segnaposto "Aggiungi tag…" `--tx2` 12.5 (`padding:3px 6px`) |
| `ColorSwatchPicker` | 6 cerchi 20 px, `gap:7px`, `padding:7px 0`; scelto con anello (§2.10). Nessuna opzione "nessun colore" disegnata |
| `FavoriteToggle` | pillola `padding:8px 12px; radius:999; bg --active`, "★" `--cred` + "Preferito" (solo lo stato attivo è disegnato) |
| `NodeIcon` / `IconTile` | tessera 52 px `radius:14 bg rgba(255,255,255,.07)` (dettaglio), 44 px `radius:12 bg --field border --bd` (editor), 52 px `bg --active` (vault) |
| `DropZone` | `padding:30px 20px; radius:14; border:1.5px dashed --bd; text-align:center; gap:8px` |
| `ProgressBar` | `height:4px; radius:4`; binario `rgba(255,255,255,.12)`; riempimento `--ac-tx` |
| `StackedBar` | `height:10px; radius:999; overflow:hidden; gap:2px`; segmenti `flex: <conteggio>` |
| `MiniBar` | `height:6px; radius:6`; binario `rgba(255,255,255,.1)`; larghezza relativa al massimo |
| `StatTile` | `padding:14px 16px; radius:12; border --bd`; etichetta 12.5 `--tx3` con pallino 8 px; valore 34/600; percentuale mono 12 `--tx2` con virgola decimale ("83,1%") |
| `ValidationRow` | griglia `76px minmax(0,1fr) auto`, `gap:3px 14px`, `padding:12px 8px`, `border-top:1px solid --bd`; gravità 600 col marcatore; percorso mono 12.5 (riga 1); messaggio `--tx3` 1.45 (riga 2); "Vai al nodo →" `--ac-tx` su due righe; hover `rgba(255,255,255,.05)` |
| `ReportRow` | griglia `104px minmax(0,1fr)`, `gap:12px`, `padding:9px 0`, `border-top --bd`; stato 12.5 (Parziale `--warn`, Non supportato `--tx3`, Saltato `--tx2`); percorso + motivo 12.5 `--tx2` |
| `CommandPreview` | `padding:14px 16px; radius:12; bg --term; border rgba(255,255,255,.08)`; didascalia 12 `--tx2` (`margin-bottom:6px`); comando mono `13px/1.6`, `overflow-wrap:anywhere` |
| `TerminalView` | `padding:16px 18px; radius:14; bg --term; border rgba(255,255,255,.08); font:13px/1.65 mono; color --tx3; cursor:text`; prompt `--ac-tx`; cursore `caret-color: --ac-tx` |
| `EmptyState` | scheda vetro `max-width:440px; padding:36px; gap:14px`, centrata |
| `VaultStatus` | piè della barra laterale (§5.1) |
| `Logo` | §1.2 |
| `BackgroundCanvas` | §3.1 |
| `WindowChrome` | tre "semafori" finti 11 px `rgba(255,255,255,.16)`, `gap:7px` (§8) |

---

## 5. Schermate e flussi

### 5.1 Finestra principale ("01 Finestra principale")
**Struttura**: canvas di sfondo; griglia con toolbar (riga 1) e corpo (riga 2) a due colonne: barra laterale
`minmax(220px,320px)` e area centrale `minmax(0,1fr)`.

**Toolbar** (vetro pannello, `padding:10px 14px`, `gap:10px`, `radius:18`, `flex-wrap:wrap`):
1. tre semafori finti (§8); 2. logo 24 px (`margin-left:10px`); 3. "portolano" (16/600); 4. nome del documento in mono
12 `--tx2` (`margin-left:6px`, ellissi), es. "infrastruttura.portolano"; 5. a destra (`margin-left:auto`, `gap:8px`):
"+ Connessione", "+ Cartella", "+ Credenziale" (bottoni toolbar) · separatore verticale 1 px `--bd` (`margin:4px 2px`)
· "Importa", "Valida", "Preferenze" (ghost) · "Connetti" (primario `8px 18px`; `opacity:.45` se la selezione non è una
connessione).
- "+ Connessione" / "+ Cartella" / "+ Credenziale" → editor del nuovo nodo (titolo "Nuova connessione SSH" / "Nuova
  cartella" / "Nuova credenziale", nome "Senza nome").
- **[CORREZIONE]** "+ Connessione" deve far scegliere **protocollo (SSH o RDP)** e **cartella di destinazione**
  (NOTE.md); il mock apre direttamente l'editor SSH. Interfaccia non disegnata *(proposta: nel passo 1 un segmented
  "SSH | RDP" e un campo "Cartella" precompilato con la cartella selezionata, con "Scegli…" che apre il picker
  filtrato sulle cartelle)*. Dove finiscono nuove cartelle e credenziali: non specificato (stessa proposta).
- "Importa" → wizard (§5.13); "Valida" → tab Validazione (§5.14); "Preferenze" → §5.16; "Connetti" → connette la
  selezione.

**Barra laterale** (vetro pannello, colonna, `overflow:hidden`):
- Testa (`padding:12px 12px 8px`, `gap:8px`): ricerca, segnaposto "Cerca per nome, host, tag…"; filtri "Tutti", "SSH",
  "RDP", "Credenziali", "★" (`gap:5px`, a capo).
- Albero (`flex:1`, scorrevole, `padding:4px 8px 8px`): `TreeRow`.
  Ricerca: confronta senza maiuscole nome, host, tipo e tag dei nodi non cartella; mostra le cartelle che contengono un
  risultato (aperte). Filtro "★" = preferiti.
  Clic: seleziona, porta alla tab dettaglio, nasconde il comando e apre/chiude se è una cartella. Doppio clic: connette
  (se c'è già una sessione per quel nodo la porta in primo piano; se era fallita riprova).
  Stati vuoti: "Nessun nodo. Le connessioni che crei o importi compaiono qui." (documento vuoto); "Nessun risultato per
  questa ricerca." (`padding:28px 16px`, centrato, `--tx2`, `line-height:1.5`).
- Piè (`padding:10px 14px`, `border-top --bd`, 12 px `--tx2`, `gap:8px`): lucchetto 14 px `--ok`, "Portachiavi di
  sistema · sbloccato" (clic → sblocco del vault), a destra "{n} nodi" in mono.
  **[CORREZIONE]** (NOTE.md): "portachiavi di sistema" solo con il backend Keychain; con il vault su file indicare il
  file cifrato e lo stato *(proposta: "Vault cifrato · bloccato" con lucchetto `--tx3`, "Vault cifrato · sbloccato"
  con lucchetto `--ok`, nome del file nel tooltip; il clic apre lo sblocco solo se bloccato; con Keychain "Portachiavi
  di sistema" senza stato né clic)*.

**Area centrale** (colonna, `gap:10px`): barra delle tab (§4.8) + una fra: stato vuoto, dettaglio, validazione,
sessione.

**Stato vuoto del documento** (scheda centrata): titolo "Il documento è vuoto"; testo "Aggiungi la prima connessione
oppure importa quelle che hai già in Royal TSX. Le password restano nel portachiavi di sistema."; bottoni "Nuova
connessione" (primario grande) e "Importa da Royal TSX" (secondario grande). Con documento vuoto la barra delle tab non
c'è. **[CORREZIONE]** la frase sul portachiavi dipende dal backend (con il file: "nel vault cifrato" *(proposta)*).

Scorciatoie (DS): ⌘↩ Connetti, ⌘E Modifica, ⌘D Duplica, ⌘⌫ Elimina. Menu contestuale: §4.15 (il trigger, clic destro
sulla riga, è implicito).

### 5.2 Dettaglio del nodo ("02 Dettaglio del nodo")
Pannello vetro, `flex:1`, scorrevole, `padding:26px 28px`, `gap:20px`.
**Testata**: tessera icona 52 px; colonna (`min-width:240px`): tipo 12 `--tx2` ("Connessione SSH", "Connessione RDP",
"Credenziale", "Cartella", "Non supportato"); nome 26/600 + "★" `--cred` + tag; riepilogo mono 14 `--tx3`
(connessione `utente@host:porta`; credenziale `DOMINIO\utente` o `utente`; cartella "N elementi" = tutti i
discendenti; non supportato "VNC · host:porta"). Azioni (`gap:8px`): "Connetti" (solo connessioni), "Modifica",
"Mostra comando"/"Nascondi comando" (solo connessioni), "Elimina" (`danger-text`).
**[CORREZIONE]** nodo non supportato: nascondere "Modifica", resta solo "Elimina" (NOTE.md).

**Anteprima comando** (con "Mostra comando"): didascalia "Comando che verrà lanciato. La password non compare: viene
letta dal portachiavi all'avvio." e il comando.
**[CORREZIONE]** il comando va preso dal **piano reale** (`plan_connection` → `render`), non composto nella UI
(NOTE.md):
- SSH (§7.2, DEC-020): `ssh -p <porta> -l <utente> [-i <chiave> -o IdentitiesOnly=yes] [-J u@h:p,…] [-A] [-X|-Y] [-C]
  [-o ServerAliveInterval=N] [-o K=V…] [-o NumberOfPasswordPrompts=1] -- <host>`, es.
  `ssh -p 22 -l deploy -J deploy@bastion.example.com:2222 -A -X -o ServerAliveInterval=30
  -o StrictHostKeyChecking=accept-new -- web01.example.com` (il mock mostra `ssh -J … -p 22 deploy@host`);
- RDP (§7.3, DEC-020): binario `sdl-freerdp` (non `xfreerdp`), `/v:host:porta /u:utente /d:dominio /cert:tofu
  /dynamic-resolution +clipboard /from-stdin:force`, gateway `/gateway:g:<host>[:<porta>],u:…,d:…`, mai `/p:`;
- la didascalia dipende dal backend (*proposta*: "viene letta dal vault all'avvio") e, per RDP, la password passa da
  stdin. Le proprietà segrete mostrano solo lo stato.

**Rotta** (solo connessioni con jump): intestazione "Rotta" e `RouteChain`.

**Credenziale effettiva** (se il nodo ne ha una): riquadro con "Credenziale effettiva" + `ModeChip` con l'origine +
testo `--tx3`; griglia `repeat(auto-fit,minmax(150px,1fr))`, `gap:16px`: "Utente" (mono 500 14), "Dominio" ("—" se
assente), "Password" (`SecretBadge`), "Chiave privata" (13.5 `--tx3`, es. "Nel portachiavi, con passphrase",
"Nessuna"). Origini del mock: "Ereditata dalla cartella Produzione" · "credenziale deploy"; "Inline" · "definita su
questa connessione"; "Da una credenziale" · "credenziale admin · ACME"; cartella: "Predefinita della cartella" ·
"credenziale deploy"; credenziale: "Questa credenziale" · "usata da 5 nodi" / "usata da 1 nodo". La traccia
dell'ereditarietà e il conteggio degli utilizzi inclusi gli ereditati sono esigenze aggiungibili (progress.md).
Origini non disegnate: Nessuna, Chiedi all'avvio, Per nome, ereditarietà che non trova nessuna credenziale.

**Informazioni** (griglia `150px minmax(0,1fr)`): "Descrizione", "Note" (solo se presenti), poi le proprietà custom in
mono; segreto: "•••••• · segreto, nel portachiavi".

### 5.3 Nodo non supportato ("08 Nodo non supportato")
Nel dettaglio, dopo la testata: riquadro (`padding:14px 16px; radius:12; border --bd; bg rgba(255,255,255,.04);
line-height:1.55; --tx3`) "Questo elemento è stato importato da Royal TSX ma Portolano gestisce solo SSH e RDP. Il
tipo originale è **VNC**. Lo conserviamo così com'è, in sola lettura, per non perdere dati: non si può aprire né
modificare." (il tipo in `--tx`, 600, va dal `source_type`); sotto, `<pre>` con l'XML grezzo (`--term`, mono
`12.5px/1.65`, `--tx3`, scorrevole). Info: "Descrizione" (es. "Importato da Royal TSX il 10 novembre 2026").
**[CORREZIONE]** niente "Modifica". Riepilogo host/porta: esigenza aggiungibile (progress.md). L'XML è testo: mostrarlo
come testo, mai come HTML.

### 5.4 Tab di sessione SSH e RDP
Testata (vetro pannello, `padding:14px 18px`, `radius:14`, `gap:12px`): riga con pallino 9 px della fase, riepilogo
mono 500 14, stato 600 nel colore della fase, spazio, "Riprova" (primario, solo se fallita), "Chiudi" / "Disconnetti"
(secondario; "Disconnetti" quando connessa); sotto, `SessionRoute`.
- **In corso**: stato "Connessione in corso · salto N di M"; al centro, 14.5 `--tx3`: "Autenticazione come {utente}.
  Password letta dal portachiavi." oppure "… Password assente: verrà chiesta."
- **SSH connesso**: stato "Connesso · {ms} ms"; `TerminalView` con "Last login: …", righe e prompt
  `utente@nome:~$`. Cliccando il terminale prende il fuoco.
- **RDP connesso**: scheda centrata (`max-width:520px; padding:28px; gap:16px`): "Sessione RDP aperta in una finestra
  esterna" (20/600, `-.01em`); "Il desktop remoto è gestito da FreeRDP. Qui vedi solo lo stato della sessione.";
  griglia `140px 1fr` 13 px: "Client" "FreeRDP (automatico)", "Risoluzione" "Adatta alla finestra · scala 100%",
  "Appunti" "Condivisi", "Durata" mono "00:00:12"; bottone "Porta in primo piano".
- **Fallita**: stato "Connessione rifiutata"; scheda (`max-width:560px; padding:26px; border:1px solid --err`):
  "Connessione non riuscita" (20/600 `--err`), "Il server ha rifiutato la connessione sulla porta indicata. Controlla
  che il servizio SSH sia attivo e che host e porta siano corretti.", riquadro mono `--term` con l'errore
  (`ssh: connect to host … port …: Connection refused`).
**[CORREZIONE]** stati unici "Connessione in corso", "Connesso", errore (§3.3): niente "salto N di M" né millisecondi.
Il testo della scheda d'errore va derivato dall'errore reale (il mock ne ha uno fisso). Il testo "Password letta dal
portachiavi" dipende dal backend.
Da DEC-027: la tab RDP deve lasciare lo spazio per il pannello RDP integrato futuro (IronRDP). Il terminale integrato
SSH è previsto in FASE 2 (DEC-013, DEC-016.2: `ssh` in una propria PTY); il mock ne simula solo l'aspetto.
Ambiguità aperte (§9): cosa fa "Porta in primo piano"; come si comporta la tab quando FreeRDP deve chiedere una
password e il piano lo vuole in primo piano con un terminale (DEC-025); durata reale; stato con Windows App.

### 5.5 Editor in due passi ("03 Editor")
`SidePanel` (§4.17). **Testata**: tessera icona 44 px (colore del tipo); sottotitolo 12 `--tx2` ("Nuova …"/"Modifica
connessione SSH|connessione RDP|credenziale|cartella"); titolo 20/600 (nome o "Senza nome"); "Annulla". **Stepper**
(§4.25). **Piede**: "Indietro" (passo 2) a sinistra, a destra "Avanti" (passo 1) o "Salva" (passo 2).
**[CORREZIONE]** "Salva" disponibile **anche al passo 1** (per cambiare solo il nome) (NOTE.md). "Salva" spento finché
non cambia qualcosa.
**[CORREZIONE]** in un nodo **nuovo** nessun valore d'esempio: niente jump host, gateway, opzioni libere precompilati
(il mock li mostra sempre).
**[CORREZIONE]** segreti in staging: "Imposta", "Sostituisci" e "Cancella" **non scrivono subito** nel vault; si
applicano con "Salva" (segreti → documento con rollback; la sostituzione usa un nuovo `SecretRef` e il vecchio si
cancella dopo il salvataggio) e si annullano con "Annulla" (NOTE.md, progress.md). La resa di un segreto "da salvare"
non è disegnata (ambiguità §9). Dopo `ModifiedExternally` le modifiche si riapplicano dopo il ricaricamento
(progress.md): stato non disegnato.

**Passo 1 · Generale** (intestazione "Generale", `gap:16px`):
- "Nome" (a fuoco all'apertura); "Descrizione"; "Note" (area di testo).
- Riga (`gap:20px`, allineata in basso): "Icona" → bottone "Scegli…" (apre §5.7); "Colore" → `ColorSwatchPicker`;
  `FavoriteToggle` "Preferito".
- "Tag" → `TagInput`.
- "Proprietà custom" → `KeyValueEditor` (es. "Ambiente" · "Testo" · "produzione"; "Token API" · "Segreto" · badge +
  "Sostituisci") e "+ Proprietà".
- Nome: rifiutato se già usato da un fratello (`NameTaken`, DEC-022.5): errore sotto il campo *(proposta di testo:
  "Esiste già un elemento con questo nome in questa cartella")*.

**Passo 2 · SSH** ("Connessione"):
- "Host" (mono) + "Porta" (mono, colonna 96 px). **[CORREZIONE]** porta lasciata al predefinito = valore assente
  (`port: None`), non il numero: campo vuoto con segnaposto "22" *(proposta)*.
- "Credenziale" → `CredentialField` (chip modalità, origine, lucchetto, "Cambia" → §5.6).
- "Jump host · in ordine di attraversamento" → `SortableList` (es. "1 bastion · Produzione / bastion · dall'albero";
  "2 deploy@198.51.100.40:22 · indirizzo libero"), "+ Dall'albero…" (→ §5.8), "+ Indirizzo libero"; catena "tu →
  bastion → 198.51.100.40 → {nome}". I jump dall'albero devono essere connessioni SSH (DEC-022.3).
- "Opzioni avanzate": toggle "Agent forwarding", "Compressione", "X11 forwarding", "X11 trusted" ("attivabile solo con
  X11 acceso").
- "Keepalive ogni [30] secondi (0 = disattivato)". **[CORREZIONE]** campo vuoto = disattivato: testo "secondi (vuoto
  = disattivato)" *(proposta)*.
- "Opzioni libere (chiave = valore)" → griglia chiave/valore mono (es. `StrictHostKeyChecking` = `accept-new`) e
  "+ Opzione". Da DEC-026: sono ammesse solo le opzioni dell'elenco chiuso (es. `ConnectTimeout`,
  `ServerAliveCountMax`, algoritmi, `StrictHostKeyChecking` solo `yes|ask|accept-new`); una chiave non ammessa è un
  errore di campo *(proposta di testo: "Opzione non ammessa: aggiungila al tuo ~/.ssh/config")*.

**Passo 2 · RDP** ("Connessione"):
- "Host" + "Porta" (come SSH, predefinito 3389).
- "Credenziale" (`CredentialField` con utente mono) + "Dominio" (colonna 150 px). **[CORREZIONE]** il dominio
  appartiene alla credenziale inline: con credenziale ereditata o collegata non esiste un dominio separato della
  connessione. Il campo "Dominio" separato va tolto; il dominio si modifica nel selettore (modalità Inline) e il
  `CredentialField` può mostrarlo in sola lettura (`DOMINIO\utente`) *(proposta)*.
- "Schermo": "Risoluzione" segmented "Adatta alla finestra | Schermo intero | Fissa" (+ L×A mono 52 px, "×", solo con
  "Fissa"); "Scala" "100% | 140% | 180%"; "Profondità colore" select ("32 bit"; valori SPEC 15/16/24/32).
- "Gateway": "Host del gateway" + "Porta" (es. 443); "Credenziale del gateway" → `CredentialField` (es. "Da una
  credenziale" · "admin · ACME" · "password assente"). Sezione chiusa finché l'host è vuoto.
- "Sessione": toggle "Multimonitor", "Sessione amministrativa", "Appunti condivisi" (predefinito acceso).
- "MAC per Wake-on-LAN" (mono; il WoL è fuori scope, il valore è solo memorizzato, SPEC §7.3/§14) e "Client"
  segmented "Automatico | FreeRDP | Windows App" (per connessione, DEC-016.7).

**Passo 2 · Credenziale** ("Accesso", senza intestazione di gruppo):
- "Username" (mono) + "Dominio" (segnaposto "Nessuno" `--tx2`) *(proposta: etichetta "Utente" come altrove)*.
- `SecretRow` "Password": badge + "Sostituisci…" + "Cancella" ("Cancella" chiede conferma; con il segreto assente
  c'è "Imposta…").
- Riquadro "Chiave privata": segmented "Nessuna | File su disco | Nel portachiavi"; con "File su disco" percorso mono
  (es. `~/.ssh/id_ed25519`) + "Sfoglia…". La resa di "Nel portachiavi" **non è disegnata** *(proposta: `SecretRow`
  con "Imposta…" che importa il contenuto da un file, mai file temporanei)*.
- `SecretRow` "Passphrase": badge "Assente" + "Imposta…".

**Passo 2 · Cartella** ("Credenziale predefinita"): `CredentialField` "Credenziale predefinita" (es. "Da una
credenziale" · "Credenziali / deploy"); nota `--tx3`: "La ereditano 4 dei 6 elementi di questa cartella. Gli altri 2
(db01, win-app01) hanno una credenziale propria." oppure "Gli elementi senza una credenziale propria usano quella
della cartella superiore, se esiste." Cambiandola, i nodi che la ereditano lampeggiano (480 ms).

### 5.6 Selettore di credenziale ("04 Selettore di credenziale")
Dialogo 820 px (App; la tavola Editor lo descrive come popover ancorato al campo: ambiguità §9) con
`CredentialSelector`; piede "Fatto" (primario). Contenuto di "Da dove arriverà":
- **Eredita**: catena "web01 → cartella Produzione → credenziale deploy"; "web01 non ha una credenziale propria. Usa
  quella predefinita della cartella più vicina che ne ha una."; griglia "Utente" `deploy` · "Password" "Nel
  portachiavi" (`--ok`) · "Chiave" "Nel portachiavi, con passphrase". Caso senza nessuna cartella con credenziale (il
  risultato è "nessuna", SPEC §4.3): **non disegnato**.
- **Nessuna**: titolo "Nessuna credenziale" (18/600); "Portolano non passa utente né password. Decide il client: per
  SSH valgono le impostazioni di ~/.ssh/config, per RDP il client chiede i dati nella propria finestra. L'ereditarietà
  dalla cartella è ignorata."
- **Chiedi all'avvio**: titolo "Te la chiediamo a ogni avvio"; "Prima di connettere si apre il dialogo con utente e
  password. Quello che inserisci vale per quella sessione e non viene salvato."; campo "Utente proposto (facoltativo)".
  **[CORREZIONE]** niente "Utente proposto" per ora (DEC-016.3, NOTE.md). Il dialogo che chiede utente (dominio per
  RDP) e password all'avvio **non è disegnato** nel mock: va progettato (DEC-016.3, `SessionCredential`).
- **Inline**: "Definita su questa connessione e non riutilizzabile altrove."; "Utente" + "Dominio" (mono);
  "Password" + badge "Assente" + "Imposta…".
- **Da una credenziale**: riga con icona credenziale, "deploy" (600), "Credenziali / deploy" (mono 12 `--tx2`),
  "Scegli…" (apre il picker con il filtro "Solo credenziali"); "Collegata al nodo: se la credenziale viene rinominata
  o spostata, il collegamento resta valido."; griglia "Utente" · "Password".
- **Per nome**: campo "Nome della credenziale" (mono, `max-width:260px`); "Oggi corrisponde a" + chip accento
  "Credenziali / deploy"; testo "Cercata per nome a ogni connessione, partendo dalla cartella del nodo e risalendo. Se
  nessuna credenziale ha questo nome, la connessione si ferma con un errore chiaro."
  **[CORREZIONE]** (SPEC §4.3, DEC-016.8): la ricerca è in **tutto il documento**, prima con maiuscole esatte poi senza;
  errore anche se il nome è **ambiguo** *(proposta di testo: "Cercata per nome in tutto il documento a ogni
  connessione. Se nessuna credenziale ha questo nome, o se più d'una lo ha, la connessione si ferma con un errore.")*;
  serve anche lo stato "nessuna corrispondenza" / "ambiguo" al posto di "Oggi corrisponde a" (non disegnato).

### 5.7 Selettore icona
Dialogo 440 px: "Scegli un'icona"; segmented a tutta larghezza "Portolano | Importate | Immagine"; campo "Cerca
icona…"; griglia `repeat(7,1fr)`, `gap:8px`, tessere quadrate `radius:10` (`bg --field`, scelta `bg --active` +
`border --ac-tx`, hover `--active` 160 ms), icona 20 px `--tx3`, `stroke-width:1.6`; piede "Annulla" / "Usa icona".
"Importate" = libreria di icone arrivata con l'import; "Immagine" = area di rilascio per PNG o SVG (non disegnata).
"La griglia è un campione ripetuto: il set definitivo di icone va disegnato a parte." Mappatura delle icone Royal sul set
builtin e `IconStore::list()`: progress.md. **Sicurezza** (DEC-024): le SVG importate si mostrano solo come `<img>`,
mai inserite nel DOM.

### 5.8 Picker di un nodo
Dialogo 440 px: "Scegli un jump host"; riga con campo di ricerca (a fuoco, es. "bast") + chip "Solo SSH" (`bg
--active`, 12 px); albero filtrato: cartelle (altezza 32, `padding:0 10px`, `--tx2`, "▾" 10 px), connessioni
(altezza 32, `padding:0 10px 0 28px`, `radius:9`, icona SSH 15 px, nome con la parte trovata in `<b>` `--ac-tx` 600,
host mono 11.5 `--tx2`; scelta `bg --active`); riepilogo 12.5 `--tx2` "2 risultati su 9 connessioni SSH. web01 è
escluso perché non può fare da salto a se stesso."; piede "Annulla" / "Scegli bastion" (l'etichetta contiene il nome).
Frecce e Invio per scegliere. Lo stesso dialogo serve per le credenziali ("Solo credenziali") e, *(proposta)*, per le
cartelle di destinazione. Stato senza risultati: non disegnato.

### 5.9 Dialogo password
Dialogo 440 px: titolo "Imposta la password" + sottotitolo mono "Credenziali / admin · ACME"; "Password" (puntini);
"Conferma" (bordo `--ok` e "Le due password coincidono" in `--ok` quando coincidono; non coincidenti: bordo `--err` e
scuotimento, testo non disegnato *(proposta: "Le due password non coincidono")*); testo 13 `--tx3` "Viene salvata nel
portachiavi di sistema. Dopo il salvataggio non sarà più visibile, nemmeno qui."; piede "Annulla" / "Salva nel
portachiavi". Nessun "mostra password".
**[CORREZIONE]** il titolo indica **il campo e il nodo** (password, passphrase, proprietà "Token API"…), non un testo
fisso (NOTE.md) *(proposta: "Imposta la passphrase", "Sostituisci «Token API»", sottotitolo = percorso del nodo)*.
**[CORREZIONE]** dall'editor il segreto è in staging: testo e bottone non possono promettere il salvataggio immediato
*(proposta: bottone "Imposta"; testo "Verrà salvata nel vault quando salvi il nodo. Dopo il salvataggio non sarà più
visibile, nemmeno qui.")*; il nome del backend dipende dalla configurazione.

### 5.10 Sblocco del vault
Dialogo 440 px centrato (`padding:26px 22px`, testo centrato): tessera 52 px `--active` con lucchetto 26 px
`--ac-tx`; "Sblocca il vault"; percorso mono 12.5 `--tx2` (es. `~/Documenti/infrastruttura.vault`); campo "Master
password" (a fuoco); due bottoni a metà larghezza "Apri senza segreti" / "Sblocca". "Compare solo se il vault è un file
cifrato." Allo sblocco il lucchetto si apre (260 ms) e il dialogo si dissolve.
"Apri senza segreti mostra l'albero ma tutte le password risultano assenti finché non si sblocca."
**[CORREZIONE]** con il vault bloccato i segreti sono "Vault bloccato", non "Assenti" (NOTE.md); "Ricontrolla" con il
vault bloccato dà una sola nota invece di un `missing_secret` per ogni password (progress.md).
Non disegnati: master password errata *(proposta: errore sotto il campo e scuotimento)*, creazione di un nuovo vault
(minimo 12 caratteri, DEC-024.7/DEC-026.6), attesa durante la derivazione della chiave.

### 5.11 Conferma eliminazione
Dialogo 460 px: titolo 'Eliminare "{nome}"?' (la tavola Dialoghi include il tipo: 'Eliminare la credenziale
"deploy"?'). Se usata: riquadro `border:1px solid --warn` "È usata da 5 nodi" (`--warn` 600) con righe 13 px
percorso / relazione `--tx2` ("Produzione" · "credenziale predefinita", "Produzione / web01" · "ereditata",
"Produzione / web02" · "ereditata", "Produzione / bastion" · "ereditata", "Staging / stg-web01" · "collegata"); ogni
riga porta al nodo. Testo 13 `--tx3` "Questi nodi resteranno senza credenziale. Password, chiave e passphrase verranno
tolte anche dal portachiavi." Se non usata: "L'elemento verrà tolto dal documento. Nessun altro nodo lo usa." Piede
"Annulla" (secondario, 600, predefinito con Invio) / "Elimina comunque" (`danger`).
**[CORREZIONE]** (DEC-016.5, DEC-018.1, NOTE.md):
- i nodi che usavano una credenziale eliminata passano a **"Chiedi all'avvio"**, non "senza credenziale" *(testo
  proposto: "Questi nodi passeranno a «Chiedi all'avvio». Password, chiave e passphrase verranno tolte anche dal
  vault." — la CLI dice "{n} nodi ora chiedono le credenziali all'avvio")*;
- il dialogo è **generico**: qualunque nodo usato da altri (credenziale, jump host come `bastion`, cartella che li
  contiene) mostra l'elenco degli utilizzi e "Elimina comunque"; i jump host eliminati restano come riferimenti
  rotti segnalati dalla validazione *(testo proposto: "La validazione segnalerà i riferimenti rimasti.")*;
- per una **cartella**: quanti elementi e quanti segreti verranno cancellati;
- senza utilizzi il bottone è "Elimina" *(proposta; il mock dice sempre "Elimina comunque")*.
Il core conta i riferimenti diretti (`references_to`: `$ref` e `ByName`); nel mock compaiono anche i nodi che
ereditano, che sono un'esigenza aggiungibile (progress.md): ambiguità §9.

### 5.12 Primo avvio
Dialogo 460 px (`padding:26px 22px`): logo 30 px + "portolano" 22/600; "Le connessioni vivono in un documento. Creane
uno nuovo o apri quello che usi già."; scheda primaria (`padding:14px 16px; radius:12; bg --ac; --on-ac`) "Crea un
nuovo documento" (600 15) / "Vuoto, pronto per la prima connessione o per un'importazione." (12.5); scheda secondaria
(`bg --field; border --bd`, hover `--active`) "Apri un documento esistente" / "Un file .portolano dal disco o da una
cartella condivisa." (12.5 `--tx3`). Sfondo a piena intensità; dopo la scelta i pannelli entrano in sequenza.
**[CORREZIONE]** togliere "o da una cartella condivisa" (SPEC §14, NOTE.md). L'estensione `.portolano` (mock) contro
`portolano.json` (SPEC) è da decidere (progress.md). **[CORREZIONE]** il primo avvio compare quando non c'è un
documento, non cliccando il logo (NOTE.md). Le schede sono `div` cliccabili: vanno rese bottoni (§7).

### 5.13 Wizard d'import (4 passi)
Dialoghi 560 px (passi 1–3) e 860 px (passo 4), `gap:16px` (report 20), stepper in testa.
**Passo 1 · File**: `DropZone` "Trascina qui il documento di Royal TSX" (16/600), "File .rtsz, oppure" (`--tx2`),
"Scegli file…"; dopo la lettura, riga file (`padding:10px 12px; radius:10; bg --field; border --bd`): nome mono
"ufficio.rtsz", "248 KB · 142 elementi letti" (12.5 `--tx2`), "Leggibile" (`--ok`, a destra). Piede "Annulla" /
"Avanti". **[CORREZIONE]** formati accettati: `.rtsz`, `.rtsx` e XML, anche compressi gzip o zip (NOTE.md, SPEC §8.3)
*(testo proposto: "File .rtsz, .rtsx o XML, oppure")*. Non disegnati: lettura in corso, file illeggibile, "Avanti"
spento senza file. L'anteprima (conteggi e numero di password) è un'esigenza aggiungibile (progress.md).
**Passo 2 · Opzioni**: "Cartella di destinazione" ("Radice del documento / Importati da Royal TSX" + "Scegli…"); "Se
un elemento esiste già": segmented "Errore | Aggiorna | Nuovi ID" (↔ `fail | update | new-ids`, SPEC §8.4) con frase
che cambia con la scelta (solo "Aggiorna" scritta: "Gli elementi già importati vengono aggiornati con i dati del file.
Le password che hai impostato in Portolano restano." — per le altre *(proposta)* "Errore": "L'importazione si ferma
se un elemento esiste già e ti mostra quali."; "Nuovi ID": "Gli elementi vengono aggiunti come nuovi, anche se esistono
già."); `ToggleRow` "Includi il cestino" / "7 elementi eliminati in Royal TSX"; `ToggleRow` "Chiedi le password" / "12
campi password da reinserire nel passo successivo" (spento → passo 3 saltato e barrato). Piede "Indietro" / "Avanti"
(`space-between`).
**Passo 3 · Password**: contatore "3 / 12" (mono 22), `ProgressBar` (25%), "2 inserite · 0 saltate"; scheda
(`grid 90px 1fr`): "Nodo" "Produzione / Database / db01", "Utente" `admin`, "Campo" "Password della connessione";
campo "Password" (sempre a fuoco); "Le password di Royal TSX sono cifrate e non si possono recuperare. Inseriscile ora,
oppure salta: potrai impostarle in seguito da ogni nodo."; bottoni "Salta" (secondario), "Salta tutte le restanti"
(ghost), "Salva e continua" (primario, a destra). Invio = salva e passa alla successiva. I segreti si scrivono nel vault
solo all'inserimento finale (SPEC §8.4: segreti → icone → documento, rollback se il salvataggio fallisce). Prompter
asincrono con totale: progress.md.
**Passo 4 · Report**: testata "Importazione completata · ufficio.rtsz" (12.5 `--tx2`), "142" (64/600), "elementi
letti"; a destra "Esporta JSON" (= `--report`) / "Vai all'albero". `StackedBar` (Importati `--ac-tx`, Parziali
`--warn`, Non supportati `--uns`, Saltati `rgba(255,255,255,.25)`); 4 `StatTile` "Importati 118 · 83,1%",
"Parziali 9 · 6,3%", "Non supportati 11 · 7,7%", "Saltati 4 · 2,8%"; colonna "Per tipo" (`MiniBar`: SSH 68, RDP 35,
Cartelle 16, Credenziali 12, Altri tipi 11) e "Avvisi" ("4 password saltate: i nodi risultano con password assente.",
"Il cestino di Royal TSX (7 elementi) non è stato incluso."); colonna "Elementi da controllare · 24" con filtri "Tutti
| Parziali | Non supportati" e `ReportRow` ("Parziale · Produzione / win-app01 · Reindirizzamento delle stampanti non
gestito, ignorato."; "Parziale · Staging / stg-web01 · Sequenza di comandi dopo il login non importata."; "Non
supportato · vnc-legacy · Tipo VNC: conservato in sola lettura."; "Non supportato · Strumenti / wiki interna · Tipo
pagina web: conservato in sola lettura."; "Saltato · Produzione / Database / db02 · Password non inserita durante
l'importazione."), link "Mostra tutti i 24". Clic su una riga porta al nodo (via `source_id`, progress.md).
**[CORREZIONE]** (SPEC §8.3, DEC-016.8, NOTE.md): una password saltata rende l'elemento **Parziale**; **Saltati** sono
solo gli elementi del cestino Royal. L'esempio "Saltato · db02 · Password non inserita" diventa "Parziale"; il conteggio
"Saltati" conta il cestino non incluso. Nota: pallino della tessera Saltati `.35` contro segmento `.25` (incoerenza
minore).

### 5.14 Validazione ("07 Validazione")
Tab dell'area centrale (vetro pannello, `padding:24px`, `gap:14px`, scorrevole). Testata: "4 problemi nel documento"
(20/600), `CountBadge` "1 errore" `--err`, "2 avvisi" `--warn`, "1 nota" `--info`, "Ricontrolla" (secondario, a
destra). Righe `ValidationRow`:
- Errore · "Archivio / old-gw" · "Il jump host fa riferimento a un nodo che non esiste più." (`dangling_ref`);
- Avviso · "Staging / stg-win01" · 'La credenziale "admin · ACME" non ha password: verrà chiesta a ogni avvio.'
  (controllo nuovo, progress.md);
- Avviso · "Produzione / db01" · "Stessa credenziale inline di db02: valuta una credenziale condivisa." (controllo
  nuovo, progress.md);
- Nota · "vnc-legacy" · "Nodo non supportato (VNC), conservato in sola lettura." (`unsupported_node`, Info).
Ogni riga ha "Vai al nodo →".
**[CORREZIONE]** (NOTE.md): **ogni** riga porta al nodo, compresa quella d'errore (nel mock non ha azione); il numero
dei problemi non è fisso (titolo, contatori e tipo della tab "N problemi" vanno calcolati, con singolare/plurale:
"1 problema/N problemi", "1 errore/N errori", "1 avviso/N avvisi", "1 nota/N note"). Livelli: Errore, Avviso, Nota
(DEC-016.6), coerenti con la CLI ("errore", "avviso", "nota"). Stato senza problemi: non disegnato *(proposta:
"Nessun problema nel documento")*. `missing_secret` è un Avviso (DEC-022.4).
Variante della tavola Dialoghi a una riga (`76px 200px 1fr auto`): superata dalla App.

### 5.15 Toast, menu e tooltip in contesto
Esempi del mock: "Password salvata nel portachiavi" (successo), "db02: connessione rifiutata · Riprova" (errore),
"web02 eliminato · Annulla". Altri messaggi da prevedere: errori di salvataggio (`ModifiedExternally`: "il documento …
è stato modificato da un altro processo: ricaricalo", DEC-016.1), errori di spostamento (§3.4).

### 5.16 Preferenze ("09 Preferenze")
Dialogo 760 px (`padding:26px`), griglia `150px minmax(0,1fr)`, `gap:18px 20px`; a sinistra intestazioni di gruppo
(`padding-top:10px`), a destra i controlli; piede "Chiudi". "Le modifiche si applicano subito, senza 'Salva'". I campi
dipendenti compaiono solo con l'opzione che li richiede.
- **Documento**: percorso mono (es. `~/Documenti/infrastruttura.portolano`) + "Sfoglia…".
- **Vault**: segmented "Portachiavi di sistema | File cifrato" (↔ `keychain | file`); con "File cifrato" percorso mono
  (es. `~/Documenti/infrastruttura.vault`) + "Sfoglia…".
- **Terminale esterno**: segmented "Terminal | iTerm | Ghostty | Comando custom" (↔ `terminal | iterm | ghostty |
  custom`); con "Comando custom" campo mono "kitty --single-instance -e {comando}". **[CORREZIONE]** segnaposto
  `{cmd}`, non `{comando}` (NOTE.md, SPEC §7.2).
- **Programma SSH**: campo mono `/usr/bin/ssh`. Da DEC-024.2/DEC-026.3: percorso assoluto a un file non scrivibile da
  gruppo o altri (anche le directory); altrimenti errore di campo *(testo da definire)*; l'errore riguarda solo SSH.
- **RDP**: "Client" "Automatico | FreeRDP | Windows App"; "Percorso di FreeRDP" (mono, es.
  `/opt/homebrew/bin/xfreerdp`; **[CORREZIONE]** binario `sdl-freerdp`, es. `/opt/homebrew/bin/sdl-freerdp`, NOTE.md;
  stesse regole di sicurezza di SSH); "Certificati del server" "Fidati al primo uso | Ignora | Rifiuta" (↔ `tofu |
  ignore | deny`).
- **Aspetto**: "Tema" "Scuro | Chiaro · in arrivo" (Chiaro non disponibile, `--tx2`); "Densità" "Comoda | Compatta";
  `ToggleRow` "Riduci movimento" / "Ferma lo sfondo e sostituisce gli spostamenti con dissolvenze. Segue anche
  l'impostazione di sistema."
Le preferenze d'aspetto vanno in un file separato da `config.toml` (progress.md, DEC-011); le altre sono `config.toml`
(SPEC §9).

### 5.17 Schermate e stati non presenti nel mock (da progettare)
Scelta di protocollo/cartella per "+ Connessione"; dialogo delle credenziali all'avvio ("Chiedi all'avvio",
DEC-016.3); creazione del vault su file (master password ≥ 12 caratteri); master password errata; documento modificato
da un altro processo (ricarica, riapplicazione delle modifiche); input "+ Indirizzo libero"; selezione della cartella
di destinazione dell'import; validazione senza problemi; menu contestuale di cartelle e credenziali; stato "Vault
bloccato" nei badge dell'editor; segreti in staging; errori di lettura del file d'import; caricamento in generale
(nessuno spinner disegnato).

---

## 6. Testi: glossario

| Concetto | Etichetta UI (mock) | CLI / core | Nota |
|---|---|---|---|
| Nodo cartella / connessione / credenziale | "Cartella", "Connessione SSH", "Connessione RDP", "Credenziale" | `folder`, `connection`, `credential` | tipo nel dettaglio |
| Nodo non supportato | "Non supportato", "non supportato" | "non supportato" | |
| Preferito | "Preferito", "★" | `favorite` | |
| Credenziale ereditata | modalità "Eredita"; origine "Ereditata dalla cartella X"; relazione "ereditata" | "ereditata" | |
| Nessuna credenziale | "Nessuna" | "nessuna" | |
| Chiedere all'avvio | "Chiedi all'avvio" | "chiesta all'avvio" | |
| Inline | "Inline" | "inline" | |
| Credenziale collegata | "Da una credenziale"; relazione "collegata" | "credenziale {percorso}" | |
| Per nome | "Per nome" | "per nome '{nome}'" | |
| Credenziale predefinita della cartella | "Credenziale predefinita" | — | |
| Segreto presente / assente | "Nel portachiavi" / "Assente", "password assente" | "presente nel vault" / "assente nel vault" | dipende dal backend (§4.9) |
| Vault bloccato | "Vault bloccato" | — | |
| Portachiavi / vault | "Portachiavi di sistema" (Keychain), "File cifrato", "vault" | `keychain`, `file`, "vault" | *proposta*: "vault" come termine generico, "portachiavi di sistema" solo per Keychain |
| Master password | "Master password" | "Master password del vault" | |
| Gravità | "Errore", "Avviso", "Nota" | "errore", "avviso", "nota" | |
| Esiti d'import | "Importati", "Parziali", "Non supportati", "Saltati" (singolari: "Parziale", "Non supportato", "Saltato") | "IMPORTATI", "PARZIALI", "NON SUPPORTATI", "SALTATI"; "importato", "parziale", "saltato" | |
| Conflitti d'import | "Errore", "Aggiorna", "Nuovi ID" | `fail`, `update`, `new-ids` | |
| Jump host | "Jump host", "salto", "dall'albero", "indirizzo libero" | `jump_hosts` | |
| Rotta | "Rotta", "tu" | — | |
| Stato di connessione | "Connessione in corso", "Connesso", "Connessione rifiutata", "Connessione non riuscita" | — | §3.3 |
| Azioni | "Connetti", "Disconnetti", "Chiudi", "Riprova", "Modifica", "Modifica…", "Duplica", "Elimina", "Elimina…", "Elimina comunque", "Annulla", "Salva", "Avanti", "Indietro", "Fatto", "Scegli…", "Cambia", "Sfoglia…", "Imposta…", "Sostituisci…", "Sostituisci", "Cancella", "Ricontrolla", "Vai al nodo →", "Mostra comando", "Nascondi comando", "Apri in terminale esterno", "Porta in primo piano", "Esporta JSON", "Vai all'albero", "Salta", "Salta tutte le restanti", "Salva e continua", "Sblocca", "Apri senza segreti", "Usa icona" | | puntini di sospensione "…" (U+2026) quando l'azione apre un dialogo |
| Creazione | "+ Connessione", "+ Cartella", "+ Credenziale", "+ Proprietà", "+ Opzione", "+ Dall'albero…", "+ Indirizzo libero", "Nuova connessione" | | |
| Campi | "Nome", "Descrizione", "Note", "Icona", "Colore", "Tag", "Proprietà custom", "Host", "Porta", "Utente" ("Username" nell'editor della credenziale: *proposta* uniformare a "Utente"), "Dominio", "Password", "Conferma", "Chiave privata", "Passphrase" | | |
| Percorsi | "Produzione / web01" (separatore " / ", senza "/" iniziale) | `/Produzione/web01` | *proposta*: mantenere la forma leggibile nella UI |
| Conteggi | "{n} nodi", "N elementi", "È usata da N nodi", "usata da 1 nodo", "N problemi" | | gestire singolare/plurale |
| Ricerca | "Cerca per nome, host, tag…", "Cerca icona…", "Aggiungi tag…" | | |

Stile dei testi: italiano, frasi brevi, seconda persona singolare ("Aggiungi…", "Te la chiediamo…"), decimali con la
virgola ("83,1%"), virgolette dritte `"…"` nel mock *(proposta: «…» o "…" tipografiche in modo uniforme)*.

---

## 7. Accessibilità e contrasto

Rapporti WCAG calcolati sui colori della palette, con il vetro composto sul fondo pieno (pannello ≈ `#201E1B`,
dialogo ≈ `#24221E`, campo su pannello ≈ `#302E2B`):

| Coppia | Rapporto | Esito (AA testo normale 4.5:1) |
|---|---|---|
| `--tx` su pannello / campo | 13.68 / 11.17 | ok |
| `--tx3` #CFC9BD su pannello / campo | 10.05 / 8.21 | ok |
| `--tx2` #A8A297 su pannello / campo / `--active` | 6.53 / 5.33 / 4.93 | ok |
| `--ac-tx` su pannello / campo | 7.99 / 6.52 | ok |
| `--warn`, `--ok`, `--info` su campo | 6.36 / 7.50 / 5.97 | ok |
| **`--on-ac` #F5F1E8 su `--ac` #2F7D5B** | **4.43** | **sotto 4.5** (bottoni primari 13 px/600, badge "Nel portachiavi" 12.5 px) |
| **`--on-ac` su `--ac-h` #38916A (hover)** | **3.43** | **insufficiente** |
| **`--err` su campo / `--active`** | **4.20 / 3.88** | **sotto 4.5** (messaggi d'errore sotto i campi) |
| **`--uns` #8A8479 su pannello / campo** | **4.46 / 3.64** | testo "non supportato" sotto soglia |
| `#1B0F0D` su `--err` (distruttivo) | 5.82 | ok |
| **Bordo `--bd` su pannello** | **1.62** | **sotto 3:1** per i contorni dei controlli (WCAG 1.4.11) |
| **Binario del toggle spento su dialogo** | **1.67** | **sotto 3:1**; il pomello #CFC9BD sul binario 5.77 regge da solo |

Sfondo animato sotto il vetro: con il blur di 6 px le linee scendono a ~1/8 dell'alfa e il testo resta leggibile
(`--tx2` ≈ 6.0:1), ma un punto grande sotto un testo piccolo porta `--tx2` a ≈ 4.3:1 e `--uns` a ≈ 3.0:1 (stima). Il
vetro al 16% è molto trasparente: *proposta* verificare su schermo e, se serve, alzare l'opacità del pannello o
l'alfa delle linee/punti dietro le aree di testo.

Proposte di correzione: scurire l'accento per il testo su accento (o usare `--on-ac` più chiaro e l'hover più scuro
invece che più chiaro), schiarire `--err` per il testo (es. verso `#EE8279`, già usato come hover) o usarlo solo con
icona, alzare il contrasto di bordi dei campi e del binario spento.

Focus e tastiera:
- Focus visibile **solo** sui campi (bordo `--ac-tx`); il terminale e la ricerca hanno `outline:none`. Bottoni, tab,
  righe dell'albero, voci del selettore, segmented, toggle, schede del primo avvio, righe di validazione e del report
  non hanno focus visibile.
- Molti elementi interattivi sono `div`/`span` con `onClick` (righe, tab, "×", voci di modalità, schede del primo
  avvio, step, "Cambia", "Sostituisci" come `<a>` senza `href`): vanno resi `button`/`a` o con `role`, `tabindex`,
  gestione di Invio/Spazio.
- Nessun attributo ARIA nel mock. Servono: `role="tree"`/`treeitem` con `aria-expanded`/`aria-selected`/`aria-level`
  (navigazione con frecce, Home/End, Invio = connetti?), `role="tablist"`/`tab`, `role="dialog"` + `aria-modal` +
  `aria-labelledby`, `role="switch"` + `aria-checked` per i toggle, `role="radiogroup"` per i segmented, `aria-live`
  per toast e stato della connessione, etichette associate ai campi (`<label for>`), testo alternativo per icone e
  pallini di stato (oggi lo stato di sessione nell'albero è solo un pallino colorato).
- Tastiera citata nel mock: ⌘↩ Connetti, ⌘E Modifica, ⌘D Duplica, ⌘⌫ Elimina; frecce e Invio nel picker; Invio = Annulla
  nella conferma d'eliminazione; Invio = "Salva e continua" nell'import. Non citati: Esc, ⌘F (ricerca), ⌘, (Preferenze),
  ⌘W (chiudi tab), cambio tab, riordino da tastiera della lista dei jump.
- Positivo: la gravità usa colore + forma + parola; i segreti usano colore + bordo pieno/tratteggiato + lucchetto
  chiuso/aperto + testo; riduci movimento è previsto e segue il sistema.

---

## 8. Elementi del mock da NON implementare

- **Dati d'esempio**: tutti i nodi (Produzione, web01, db01, bastion, deploy, admin · ACME, vnc-legacy…), host
  `*.example.com` e indirizzi RFC 5737, `infrastruttura.portolano`, `ufficio.rtsz` e tutti i numeri del report (142,
  118, 9, 11, 4, 24, 12, 7, 68, 35…), "2 risultati su 9 connessioni SSH", "La ereditano 4 dei 6 elementi…", "È usata
  da 5 nodi", MAC `00:00:5E:00:53:2A`, `rdgw.example.com:443`, `kitty --single-instance -e …`, percorsi di esempio.
- **Valori precompilati nell'editor di un nodo nuovo** (jump host, gateway, opzioni libere, toggle accesi) (NOTE.md).
- **Il clic su "portolano" che apre il primo avvio** (NOTE.md) e il clic sul piè del vault con Keychain.
- **Terminale finto**: righe "Last login: …", comandi simulati (`uptime`, `whoami`, `hostname`, `ls`, `pwd`,
  `exit`, "command not found").
- **Simulazione della connessione**: timer di 750 ms per salto, latenza casuale (`18 + rand×40` ms), flag `fail` di
  db02, testo d'errore fisso, "Connesso · N ms", "salto N di M".
- **Valori fissi della scheda RDP**: "FreeRDP (automatico)", "Adatta alla finestra · scala 100%", "Condivisi", "Durata
  00:00:12" (vanno dalla connessione reale; la durata deve scorrere).
- **Comando composto nella UI** (`ssh -J … deploy@…:2222`, `xfreerdp …`): va preso dal piano.
- **Validazione statica** (4 righe e "4 problemi" fissi) e dialogo d'eliminazione legato a `deploy`.
- **Testo e XML del nodo non supportato fissi** ("VNC", `<RoyalVNCConnection>…`).
- **Griglia icone** di 21 tessere (7 icone ripetute): il set va disegnato.
- **Semafori finti** nella toolbar: in Tauri si usano le decorazioni native o `titleBarStyle: overlay` con i semafori
  di sistema (scelta da fare: ambiguità §9).
- **Tweaks del prototipo**: palette `2a`/`2b`/`1b`, intensità dello sfondo 0–10, "albero vuoto" (simulazione dello
  stato vuoto). Densità e riduci movimento invece sono nelle Preferenze e restano.
- **Sezione "Note su interazioni e animazioni"** in fondo alla tavola App (fondo `#0B0C0C`, testo `#E6E6E3`/`#D0D2CC`/
  `#A3A6A0`): è documentazione, non UI.
- **Tavole di presentazione**: intestazioni "Schermate 3 e 4 · stile 2a Grafite", sezioni larghe 2560–3220 px con
  `border-radius:36px`, costellazioni statiche con seme, didascalie sotto ogni componente, ancore `#d3`/`#e5`.
- **Tavola Brand**: direzioni 1a/1b/1c/2b, mini-finestre animate a ciclo (pronto → in corso → connesso → fallito), font
  Geist/Geist Mono/Schibsted Grotesk/Martian Mono (non vanno inclusi), motto (finché non deciso).
- **Runtime del prototipo** (`support.js`, `<x-dc>`, `sc-for`, `sc-if`, `hint-placeholder-*`, `data-screen-label`).
- **Riordino dell'albero del prototipo** (sposta il blocco prima del bersaglio cambiando profondità): il vero
  spostamento passa da `move_node` del core.

---

## 9. Ambiguità del mock che richiedono una decisione

1. **Nomi dei token di testo invertiti** (App `--tx2` = `#A8A297`, DS `--pl-text-2` = `#CFC9BD`): scegliere uno schema.
2. **Raggio 14 px** usato in 16 punti ma assente dalla scala DS.
3. **Logo**: variante Brand (6 vertici, 5 vuoti) o variante App (4 pieni) — e icona dell'app non disegnata.
4. **Velo dei dialoghi**: 45% (App) contro "si scurisce al 40%" (Dialoghi); di quanto rallenta lo sfondo con un dialogo.
5. **Riduci movimento**: durate a 0 ms (CSS della DS) o dissolvenze al posto degli spostamenti (testo della preferenza).
6. **Selettore di credenziale**: dialogo modale (App) o popover ancorato al campo (Editor).
7. **Bottone disattivato**: `opacity:.45` (toolbar) o `bg --field` + `--tx2` (DS).
8. **Densità compatta**: solo altezza 26 px (App) o anche testo 12.5 px (DS).
9. **Righe di validazione**: due righe (App) o una riga a 4 colonne (Dialoghi) — si propone la App.
10. **"+ Connessione"**: come scegliere protocollo e cartella (non disegnato); dove nascono nuove cartelle e credenziali.
11. **Resa dello stato unico di connessione** al posto dell'accensione per salto (§3.3).
12. **Tab RDP**: significato di "Porta in primo piano"; comportamento quando FreeRDP deve chiedere una password e il
    piano lo vuole in primo piano con terminale (DEC-025), dentro l'app senza terminale; stato con Windows App.
13. **Eliminazione**: l'elenco "usata da N nodi" include i nodi che ereditano (mock) o solo i riferimenti diretti del
    core (`references_to`, DEC-018.1)?
14. **Segreti in staging**: come appare un segreto impostato ma non ancora salvato; etichetta del bottone del dialogo
    password.
15. **Colore del nodo**: dove si vede (albero? icona?) e come si mostra un colore importato fuori dai 6 campioni; manca
    "nessun colore".
16. **Estensione del documento** `.portolano` contro `portolano.json`.
17. **Toast d'errore** "finché non vengono chiusi" senza bottone di chiusura disegnato; posizione e larghezza non date.
18. **Finestra**: dimensione minima; barra del titolo nativa o personalizzata.
19. **Palette**: solo 2a (le Preferenze non offrono la scelta) o anche 2b come tema alternativo; intensità dello sfondo
    fissa a 8?
20. **Piè del vault con Keychain**: ha senso "sbloccato"? (il backend Keychain non ha uno stato gestito dall'app).
21. **Cartella chiusa nell'albero**: conteggio degli elementi nella meta (DS) o niente (App).
22. **Hanken Grotesk e `tabular-nums`**: verificare la feature nel file incluso.
23. **Eredita senza risultato** e **Per nome ambiguo/non trovato**: stati non disegnati.
24. **Terminale integrato**: componente di emulazione non definito dal mock (solo l'aspetto).
