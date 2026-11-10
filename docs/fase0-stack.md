# FASE 0 – Scelta dello stack

## Vincoli che decidono la scelta

1. **Multipiattaforma, macOS per primo** (Apple Silicon).
2. **SSH**: autenticazione con password o chiave (anche con passphrase), jump host; terminale esterno ora,
   **terminale integrato** nella FASE 2.
3. **RDP**: nessun ecosistema ha una libreria RDP matura *e* facile. Le opzioni realistiche sono:
   - **client esterno**:
     - Microsoft *Windows App*, tramite un file `.rdp` generato
     - **FreeRDP 3** (`sdl-freerdp3`, disponibile con brew alla 3.x)
   - **[IronRDP](https://github.com/Devolutions/IronRDP)** (Devolutions): client RDP in Rust puro, con binding
     .NET e un client web/WASM. È l'unica strada concreta per un **RDP integrato** nella finestra in FASE 2.
4. **Keychain** per i segreti, con equivalenti su Windows e Linux.
5. **Core come libreria** separata da CLI e UI, con connettori e importer come moduli.

## Fatti già verificati sulla macchina

| Strumento | Stato |
|---|---|
| OpenSSH | 10.3: supporta `-J` (jump host) e `SSH_ASKPASS_REQUIRE=force` |
| .NET SDK | 8, 9, 10 installati |
| Swift | 6.4, ma solo Command Line Tools (**Xcode assente**, serve per un'app SwiftUI) |
| Rust, Go | **non installati** (si installano con brew/rustup in pochi minuti) |
| Node | 20 |
| FreeRDP, Windows App | non installati |

## Connettori: scelte indipendenti dal linguaggio

- **SSH (FASE 2a)**: si lancia l'**OpenSSH di sistema**, senza reimplementare il protocollo.
  `ssh -p <port> -l <user> [-i <key>] [-J jump1,jump2] <host>`, nel terminale corrente o in una nuova finestra
  (Terminal / iTerm / Ghostty, configurabile).
  - La password **non** passa mai per argv, variabili d'ambiente o file. `SSH_ASKPASS` punta al nostro stesso
    binario in modalità `askpass`, che legge il segreto dal Keychain e lo scrive su stdout verso ssh.
  - La passphrase delle chiavi passa per lo stesso askpass, oppure per ssh-agent.
  - Vantaggi: compatibilità totale con `~/.ssh/config`, `known_hosts`, agent, FIDO e così via.
- **SSH (FASE 2, terminale integrato)**: PTY che esegue lo stesso `ssh`, visualizzato da un emulatore VT
  (xterm.js o SwiftTerm). La stessa logica di lancio serve sia la CLI sia la UI.
- **RDP (FASE 2a)**: due backend dietro la stessa interfaccia:
  - **FreeRDP** (`sdl-freerdp3`): password via **stdin** (`/from-stdin`), quindi autologon vero come in Royal;
    supporta `/d:` (dominio), `/g:` (gateway), `/size:`, `/scale:`.
  - **Windows App**: si genera un `.rdp` temporaneo (host, utente, dominio, gateway, risoluzione) e lo si apre
    con `open`. È l'esperienza più curata su macOS, ma **non accetta la password** dal file: la chiede, o la
    prende dal proprio portachiavi.
  - **Default proposto**: FreeRDP se installato (autologon), altrimenti Windows App. Si può forzare per connessione.
- **Vault**: Keychain di macOS (service `portolano`, account = UUID). Su Windows e Linux si usano i backend
  nativi (Credential Manager, Secret Service). Come ripiego c'è un **vault su file cifrato** (Argon2id +
  XChaCha20-Poly1305, master password) per le macchine senza portachiavi.
- **Persistenza**: **JSON** (un file per documento, chiavi ordinate, formattazione stabile), così è
  **versionabile con git e leggibile nei diff**. Ha un campo `schema_version` e una catena di migrazioni
  `vN → vN+1` testate. SQLite l'ho scartato perché non è diffabile, e la mole di dati (centinaia di nodi)
  non lo giustifica. Le icone custom vanno in `icons/<sha256>.<ext>` accanto al file.

## Confronto stack

| | **Rust + Tauri v2** | **Go + Wails** | **.NET + Avalonia** | **Swift (solo macOS)** |
|---|---|---|---|---|
| Multipiattaforma | ✅ mac/win/linux | ✅ | ✅ | ❌ **requisito violato** |
| Core come libreria pulita | ✅ workspace di crate | ✅ package | ✅ progetti/assembly | ✅ SwiftPM |
| SSH | `ssh` di sistema; `russh` se serve nativo | `ssh` di sistema; **x/crypto/ssh** eccellente | `ssh` di sistema; SSH.NET | `ssh` di sistema; SwiftNIO SSH |
| Terminale integrato (FASE 2) | ✅ xterm.js + `portable-pty` (strada collaudata) | ✅ xterm.js + pty | ⚠️ nessun emulatore VT maturo per Avalonia, servirebbe una WebView con xterm.js | ✅ **SwiftTerm** (ottimo) |
| RDP integrato (FASE 2, opzionale) | ✅ **IronRDP nativo** | ❌ solo cgo verso FreeRDP | ⚠️ binding .NET di IronRDP (FFI) | ⚠️ FreeRDP via C |
| Keychain | ✅ crate `keyring` / `security-framework` | ⚠️ `go-keyring` (su mac usa la CLI `security`) o cgo | ⚠️ P/Invoke su Security.framework da scrivere | ✅ nativo |
| UI FASE 2 | Web (TS + Svelte/React): albero, drag&drop, tab facili | Web, uguale | XAML nativo-ish; TreeView e drag&drop buoni | SwiftUI/AppKit, la più nativa |
| Dimensione app | ~10–15 MB | ~10–15 MB | ~60–90 MB self-contained | ~5 MB |
| Toolchain già presente | ❌ (rustup) | ❌ (brew) | ✅ | ⚠️ manca Xcode |
| Curva / velocità di sviluppo | più ripida, compilazione lenta | la più semplice | familiare se usi C# | buona, ma solo mac |
| Affinità con Royal | – | – | Royal è .NET (date e formati simili, nessun vantaggio sostanziale) | – |

## Raccomandazione: **Rust + Tauri v2**

1. È l'**unico stack** che copre bene tutti i requisiti della FASE 2: terminale integrato con xterm.js,
   **RDP integrato possibile** con IronRDP nativo, Keychain con crate maturi, app piccola e multipiattaforma.
2. La **separazione core/UI** viene naturale. Un workspace Cargo con:
   - `portolano-core`: modello, persistenza, migrazioni, vault (trait)
   - `portolano-connectors`: `ssh`, `rdp`, con il trait `Connector` e un registro
   - `portolano-import`: `royal`, con il trait `Importer`
   - `portolano-cli`: il binario `portolano`

   Tauri in FASE 2 è solo un altro crate che dipende da `core`, senza toccare il resto.
3. Con `serde` e `quick-xml` l'importer XML e il modello JSON versionato sono semplici e ben testabili.
4. Svantaggi accettati: toolchain da installare, compilazione più lenta, UI in TypeScript (due linguaggi).

**Seconda scelta: .NET 10 + Avalonia.** Ha senso se preferisci restare su C# (toolchain già installata).
Il prezzo è un terminale integrato più difficile in FASE 2 (WebView con xterm.js) e il codice per il
Keychain da scrivere a mano. **Go + Wails** è valido per la semplicità, ma chiude la porta all'RDP
integrato. **Swift** l'ho escluso perché macOS-only.

## Piano di massima delle fasi successive (dettaglio all'inizio di ogni fase)

- **FASE 1**: workspace Cargo; modello + JSON + migrazioni; vault Keychain + vault su file; CLI
  `tree/list/add/edit/move/delete/cred`; test unitari (round-trip, migrazioni, invarianti dell'albero
  come niente cicli o ordinamento stabile, vault con backend mock).
- **FASE 2a**: `portolano connect <percorso>`; connettori SSH (askpass) e RDP (FreeRDP / Windows App);
  importer Royal con report `importati / parziali / non supportati` e confronto dei conteggi sui campioni.
