# Ricognizione tecnica: app desktop Portolano con Tauri v2 + React + TypeScript

Data della verifica: 2026-11-11/12. Macchina: macOS 27.0.1 (26A434), Apple Silicon.

**Metodo.** Le versioni vengono dalle API dei registri (`https://crates.io/api/v1/crates/<nome>` per i campi
`max_stable_version`, date e MSRV; `npm view <pkg> version|peerDependencies|engines|dist-tags`). Le API vengono dalla
documentazione ufficiale (v2.tauri.app, docs.rs) e dal sorgente su GitHub (tauri, plugins-workspace, wezterm/pty).
Non ho installato nulla e non ho toccato il repo. La ricerca web generica (WebSearch) non era disponibile, quindi ogni
fonte citata è una pagina letta direttamente. Il simbolo **[da verificare]** segna ciò che non ho potuto confermare
da una fonte primaria e che va provato in un prototipo.

## Stato attuale della macchina (verificato)

| Strumento | Stato |
|---|---|
| Rust | `rustc 1.99.1`, `cargo 1.99.1` (rustup da brew, keg-only: `PATH=/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH`, DEC-001) |
| Xcode | solo **Command Line Tools** 27.2 (`/Library/Developer/CommandLineTools`, SDK 27.2), niente Xcode.app |
| Node | via **nvm**: predefinito `v20.19.2` (npm 10.8.2); installati anche `v22.23.2`, `v20.16.0`, `v18.20.8`; `/usr/local/bin/node` = v20.16.0 |
| Gestori di pacchetti JS | npm, yarn (`/opt/homebrew/bin/yarn`); pnpm e bun assenti |
| Homebrew | 7.0.13; formula `node` non installata (brew offre `node` 26.13.0 e `node@24` 24.22.2) |
| Tauri CLI | assente (`cargo-tauri`, `create-tauri-app` non installati) |
| Binario CLI | `target/release/portolano` = **7,07 MB** (7.074.144 byte), `target/debug/portolano` = 27 MB |

Linee di Node a oggi (`nodejs.org/dist/index.json`): **24.22.3 "Krypton" (Active LTS)**, 26.13.1 "Lithium" (LTS dal
2026-10), 22.23.5 "Jod" (Maintenance LTS). **Node 20 non basta più**: Vitest 5.1 chiede `^22.12.0 || ^24 || >=26`,
jsdom 31 chiede `^22.22.2 || ^24.15.0 || >=26`, Vite 8 chiede `^20.19.0 || >=22.12.0`.

---

## 1. Tauri v2 nel workspace

### Versioni (stabili, verificate)
| Pacchetto | Versione | Data | Note |
|---|---|---|---|
| `tauri` (crate) | **2.12.5** | 2026-10-21 | MSRV 1.90 |
| `tauri-build` | **2.7.4** | 2026-10-21 | |
| `tauri-utils` | 2.10.2 | 2026-10-14 | dipendenza transitiva |
| `@tauri-apps/api` | **2.12.4** | 2026-10-14 | dist-tag `latest` |
| `@tauri-apps/cli` | **2.12.4** | | dist-tag `latest`; `next` = 3.0.0-alpha.6 |
| `wry` / `tao` | 0.57.2 / 0.37.2 | | transitive |

**Attenzione: esiste Tauri 3 in alpha.** `cargo search tauri` mostra `3.0.0-alpha.6`, e lo stesso vale per molti
plugin. Bisogna fissare le dipendenze a `2.x` (in `Cargo.toml` con `"2.12"` non si prende una pre-release, ma
`cargo add` senza versione e `npm i @tauri-apps/cli@next` sì). Sul blog l'ultimo annuncio è "Announcing Tauri 2.12"
(2026-09-26) e non c'è un annuncio di Tauri 3: **raccomando di restare su 2.12.x**.

### Struttura consigliata
```
Portolano/
├── Cargo.toml                       # members = ["crates/*"]: il crate dell'app entra da solo
├── crates/
│   ├── portolano-app/               # = "src-tauri"
│   │   ├── Cargo.toml               # [lib] name = "portolano_app_lib"; [[bin]] portolano-app
│   │   ├── build.rs                 # tauri_build::build() (o try_build con AppManifest, §6)
│   │   ├── tauri.conf.json
│   │   ├── capabilities/main.json
│   │   ├── icons/                   # icone dell'app (generate con `tauri icon`)
│   │   ├── binaries/                # sidecar portolano-aarch64-apple-darwin (non versionato, §5)
│   │   └── src/{main.rs, lib.rs, state.rs, error.rs, commands/*, terminal/*, watch.rs, icons.rs}
│   └── …crate esistenti
└── frontend/                        # progetto Vite + React + TS (package.json, src/, vite.config.ts)
```
- Va bene `frontend/` e non `ui/`, per non confonderlo con `docs/ui/` (il mock).
- La documentazione ufficiale ammette `src-tauri` come membro di un workspace Rust ("use the `src-tauri/` folder …
  as a member of your Rust workspace"). Lo split `lib.rs` (con `#[cfg_attr(mobile, tauri::mobile_entry_point)]`) e
  `main.rs` → `portolano_app_lib::run()` è quello del template; lo terrei anche senza mobile, perché costa poco.
- **La CLI di Tauri trova `tauri.conf.json`** cercando *verso il basso* da `TAURI_APP_PATH` (o dalla cwd), con
  profondità 3 (`TAURI_CLI_CONFIG_DEPTH`), e `package.json` da `TAURI_FRONTEND_PATH` (fonte: `tauri-cli/src/helpers/app_paths.rs`).
  Con `@tauri-apps/cli` come devDependency di `frontend/`, gli script npm conviene scriverli in modo esplicito:
  `"tauri": "TAURI_APP_PATH=../crates/portolano-app TAURI_FRONTEND_PATH=. tauri"`.
- Il nome del binario principale dentro il `.app` è quello di cargo (`portolano-app`), salvo `mainBinaryName`.
  **Non va chiamato `Portolano`**, perché APFS di default non distingue le maiuscole e `Contents/MacOS/Portolano`
  andrebbe in collisione con il sidecar `Contents/MacOS/portolano` (§5).
- Lint del workspace: `unsafe_code = "deny"` e `missing_docs` valgono anche per il nuovo crate. Le macro di Tauri
  (`generate_context!`, `#[tauri::command]`) generano codice che potrebbe far scattare `missing_docs` o i lint di
  clippy; se succede si mette un `#[allow]` mirato e motivato. **[da verificare]**

### `tauri.conf.json` minimo per macOS (proposta)
```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Portolano",
  "version": "0.1.0",
  "identifier": "dev.portolano.desktop",
  "build": {
    "devUrl": "http://localhost:5173",
    "frontendDist": "../../frontend/dist",
    "beforeDevCommand": { "script": "npm run dev", "cwd": "../../frontend" },
    "beforeBuildCommand": { "script": "npm run build", "cwd": "../../frontend" }
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [{
      "label": "main", "title": "Portolano", "width": 1280, "height": 800,
      "minWidth": 900, "minHeight": 560,
      "titleBarStyle": "Overlay", "hiddenTitle": true,
      "trafficLightPosition": { "x": 16, "y": 18 },
      "transparent": true,
      "windowEffects": { "effects": ["liquidGlassRegular", "sidebar"], "state": "followsWindowActive" }
    }],
    "security": {
      "csp": { "…": "vedi §6" },
      "devCsp": { "…": "vedi §6" },
      "freezePrototype": true,
      "assetProtocol": { "enable": false }
    }
  },
  "bundle": {
    "active": true,
    "targets": ["app", "dmg"],
    "icon": ["icons/icon.icns", "icons/icon.png"],
    "macOS": { "minimumSystemVersion": "14.0", "signingIdentity": "-", "hardenedRuntime": true }
  }
}
```
- L'`identifier` non deve finire in `.app`: dalla documentazione ricordo un avviso della CLI su macOS per questo caso
  **[da verificare]**. Il servizio Keychain resta `dev.portolano` (SPEC §5.2), indipendente dall'identifier.
- Lo schema (`/reference/config/`) conferma: `titleBarStyle` ∈ `Visible|Transparent|Overlay`; `hiddenTitle`;
  `trafficLightPosition` (richiede `Overlay` e `decorations: true`); `windowEffects` (richiede `transparent`);
  `app.macOSPrivateApi` è un **no-op da Tauri 2.12.1** ("the APIs are always enabled now");
  `bundle.macOS.minimumSystemVersion` vale "10.13" per default; `hardenedRuntime` è `true` per default;
  `bundle.targets` accetta `app` e `dmg`.

### Dipendenze di sistema su macOS
La documentazione dice: "If you're only planning to develop desktop apps and not targeting iOS then you can install
Xcode Command Line Tools instead" (`xcode-select --install`). **Le CLT 27.2 già presenti bastano.** Rust c'è già.
Serve solo un Node ≥ 22.12 (meglio 24 LTS, vedi sopra) e i pacchetti npm locali. **Con brew non serve nulla**: la DMG
si crea con `hdiutil` e lo script `bundle_dmg.sh` incluso nel bundler di Tauri.

**Fonti:** https://v2.tauri.app/start/prerequisites/ · https://v2.tauri.app/start/project-structure/ ·
https://v2.tauri.app/reference/config/ · https://v2.tauri.app/start/frontend/vite/ ·
https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-cli/src/helpers/app_paths.rs · https://v2.tauri.app/blog/

**Rischi:** si può installare per errore una dipendenza 3.0-alpha; ci possono essere attriti fra i lint del
workspace e le macro di Tauri; la ricerca del `tauri.conf.json` dipende dalla cwd.

---

## 2. Comandi tipizzati Rust → TypeScript

### Le opzioni
| | `tauri-specta` + `specta` | `ts-rs` | `taurpc` |
|---|---|---|---|
| Versione | **2.0.0-rc.28** (entrambi, 2026-11-05/11); `specta-typescript` **0.0.15** | **12.0.1** (2026-01-31, stabile) | 0.8.2 |
| Cosa genera | tipi **e** funzioni dei comandi, eventi tipizzati, `Channel<T>` | solo tipi (`#[derive(TS)]`, export tramite test) | proxy RPC su specta |
| Tauri v2 | sì (dipende da `tauri ^2`) | indipendente da Tauri | sì |
| Maturità | RC da molto tempo (rc.23 a marzo 2026 → rc.28 a novembre 2026, ~850k download recenti); la doc chiede di fissare `=2.0.0-rc.28` | stabile, ~10,5M download recenti | piccolo (~2,7k download) |
| Errori | `Builder::error_handling`: `Throw` (predefinito), `Result` (`{status:"ok",data}`/`{status:"error",error}`), `ResultUnknown`, `DataError` | i tipi d'errore li esporti tu, l'invoke lo scrivi a mano | |

Le feature di specta rc.28 comprendono `uuid`, `chrono`, `serde_json`, `toml`, `derive` (verificato su crates.io),
quindi `Uuid` e le date del modello si esportano senza adattatori.

### Raccomandazione: **tauri-specta 2.0.0-rc.28 + specta 2.0.0-rc.28 + specta-typescript 0.0.15, fissati con `=`**
Motivi: un'unica fonte di verità per comandi, argomenti, risultati, errori, eventi e canali; niente stringhe
`invoke('nome')` scritte a mano; i bindings si rigenerano e si controllano in CI. Il rischio della RC (API che
cambia fra una rc e l'altra) è contenuto: la versione è fissata, e l'aggiornamento diventa un lavoro esplicito.
Il ripiego, se tauri-specta si bloccasse, è ts-rs per i tipi più un modulo `ipc.ts` scritto a mano: la forma dei
DTO non cambia.

Dove vanno i derive:
- **Non** aggiungere `specta` a `portolano-core` in modo obbligatorio. O si usa
  `#[cfg_attr(feature = "specta", derive(specta::Type))]` dietro una feature opzionale di core/ops, oppure, meglio,
  si scrivono **DTO nel crate dell'app** (`TreeNodeDto`, `NodeDetailsDto`, …). I DTO sono anche il punto in cui si
  garantisce che **nessun segreto e nessun `SecretRef` arrivino al frontend** (solo `has_password: bool`). Un
  `SecretRef` non è un segreto, ma non serve alla UI.
- I tipi di input di ops (`ConflictPolicy`, `DeleteOptions`, `Placement`, `TypeFilter`, `CredentialInput`) hanno
  bisogno di `Deserialize`, come annotato in `progress.md`. Con i DTO, invece, la conversione avviene nell'app.

### Errore per l'IPC (proposta)
`OpsError` resta senza `Serialize` (ops non deve dipendere da serde per questo). L'app definisce:
```rust
/// Error returned by every command. `message` is the Italian Display of the
/// source error, which never contains a secret (SPEC §1.1).
#[derive(Debug, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct IpcError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ErrorDetails>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    NotFound, Ambiguous, Invalid, Vault, VaultLocked, Connect,
    ModifiedExternally, DocumentMissing, Referenced, FolderNotEmpty,
    ImportConflicts, Cancelled, Internal,
}

#[derive(Debug, serde::Serialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ErrorDetails {
    Referenced { references: Vec<ReferenceDto> },
    ImportConflicts { conflicts: Vec<ConflictDto> },
}

impl From<OpsError> for IpcError {
    fn from(error: OpsError) -> Self {
        let code = classify(&error); // same mapping as the CLI exit codes (DEC-022.7)
        let details = match &error {
            OpsError::Referenced { references, .. } => Some(ErrorDetails::Referenced {
                references: references.iter().map(ReferenceDto::from).collect(),
            }),
            _ => None,
        };
        IpcError { code, message: error.to_string(), details }
    }
}
```
**Suggerimento architetturale:** la classificazione oggi sta nella CLI (`portolano-cli/src/failure.rs`, che mappa gli
errori sui codici d'uscita DEC-022.7). Conviene spostarla in ops (`OpsError::category() -> ErrorCategory`) e usarla
sia nella CLI (codice d'uscita) sia nell'app (`ErrorCode`), così le due non divergono.

### Esempio minimo di comando
```rust
#[tauri::command]
#[specta::specta]
async fn node_details(
    state: tauri::State<'_, AppState>,
    id: uuid::Uuid,
) -> Result<NodeDetailsDto, IpcError> {
    let session = state.session()?;                 // MutexGuard, never held across .await
    let doc = &session.document;
    let node = doc.find(id).ok_or(IpcError::not_found(id))?;
    Ok(NodeDetailsDto::from_node(doc, node, &state.vault_presence()))
}

pub fn run() {
    let specta = tauri_specta::Builder::<tauri::Wry>::new()
        .error_handling(tauri_specta::ErrorHandlingMode::Result)   // nome esatto [da verificare]
        .commands(tauri_specta::collect_commands![node_details, open_document /* … */]);
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(specta.invoke_handler())
        .setup(move |app| {
            specta.mount_events(app);
            app.manage(AppState::new(app.handle())?);
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| { eprintln!("{error}"); std::process::exit(1) });
}
```
TypeScript generato (in modalità `Result`):
```ts
const r = await commands.nodeDetails(id);
if (r.status === "error") { showError(r.error.code, r.error.message); return; }
render(r.data);
```
- L'export dei bindings (`specta.export(specta_typescript::Typescript::default(), "../../frontend/src/ipc/bindings.ts")`)
  lo metterei in un **test** (`#[test] fn bindings_are_up_to_date`) più `git diff --exit-code` nel quality gate,
  invece che a ogni avvio in debug come negli esempi.
- Tauri richiede che anche gli errori siano `Serialize` ("everything returned from commands must implement
  serde::Serialize, including errors").
- **I comandi sincroni girano sul main thread** ("Commands without the async keyword are executed on the main thread
  unless defined with #[tauri::command(async)]"). Le operazioni lente (Argon2 da 64 MiB, Keychain, import, `save`)
  vanno in comandi `async` con `tauri::async_runtime::spawn_blocking`.

**Fonti:** https://github.com/specta-rs/tauri-specta · https://docs.rs/tauri-specta/2.0.0-rc.28/tauri_specta/ ·
https://crates.io/crates/ts-rs · https://v2.tauri.app/develop/calling-rust/

**Rischi:** tauri-specta è ancora una RC dopo anni (ogni rc può rompere qualcosa) ⇒ versione fissata e
aggiornamenti voluti; un `Channel` può stare solo come argomento diretto di un comando, non dentro struct o eventi.

---

## 3. Stato condiviso, eventi e canali

### Stato
- `app.manage(T)` richiede `T: Send + Sync + 'static`; Tauri lo mette già in un `Arc` ("you don't need to use Arc").
  La doc dice che il `std::sync::Mutex` va bene se il guard non attraversa un `.await`. Un tipo non gestito fa
  fallire il comando a runtime ("state not managed"), mentre `Manager::state()` va in **panic**: conviene un solo
  tipo `AppState`.
- Bound verificati nel codice: `trait Vault: Send + Sync`; `LazyVault` contiene `Mutex<Option<Box<dyn Vault>>>` e
  `Box<dyn MasterPasswordSource + Send + Sync>`, quindi è `Send + Sync`; `Store` ha un `Mutex<Fingerprint>` e il
  commento dice "The store is `Send + Sync`, but threads sharing one must serialize load, modify and save"; i
  trait `Protocol` e `Importer` sono `Send + Sync`, quindi i due `Registry` si possono condividere; `Document` è
  fatto di soli dati.

Proposta:
```rust
pub struct AppState {
    session: Mutex<Option<OpenDocument>>,     // None = nessun documento aperto
    vault: LazyVault,                          // backend dalla config; master password da UnlockSlot
    unlock: Arc<UnlockSlot>,                   // Mutex<Option<Secret>>, condiviso con la MasterPasswordSource
    config: RwLock<AppConfig>,
    protocols: portolano_connectors::Registry,
    importers: portolano_import::Registry,
    terminals: Mutex<HashMap<TerminalId, TerminalSession>>,
    watcher: Mutex<Option<DocumentWatcher>>,   // §10
}
pub struct OpenDocument { store: Store, document: Document }
```
**Regola:** una sola `Mutex` sulla coppia `store + document`, così `load → modifica → save` sono serializzati
(è quello che chiede il commento di `Store`). Le funzioni di ops lavorano già su una copia e sostituiscono il
documento solo se l'operazione riesce (DEC-022).

### Vault e master password nella UI
`LazyVault` chiama `MasterPasswordSource::master_password()` **in modo sincrono** quando apre il vault. Schema
consigliato:
1. `UnlockSlot` implementa `MasterPasswordSource`: se lo slot è vuoto restituisce
   `VaultError::MasterPasswordUnavailable`, che l'app mappa su `ErrorCode::VaultLocked`;
2. a quel punto il frontend mostra il dialogo e chiama `unlock_vault(password)`. Il comando mette la password nello
   slot come `Secret`, forza l'apertura (per esempio con `vault.contains(&SecretRef::new())`) e **svuota lo slot**;
3. per "Blocca vault" manca un metodo: va aggiunto `LazyVault::lock()` (svuota `opened`). È una piccola modifica a ops.

Lo stesso schema "richiesta → risposta" vale per i callback sincroni dell'import (password di Royal, "3 / 12",
"Salta tutte"): il comando gira in `spawn_blocking`, invia una richiesta al frontend su un `Channel` e si blocca su un
canale `std::sync::mpsc` finché arriva `answer_import_prompt(id, risposta)`. In alternativa si usa l'API in due
fasi già prevista da DEC-022.

**Il segreto passa dalla webview (in ingresso).** La master password e le password dell'editor si digitano in un
`<input>` e vivono per un po' come stringhe JS, che non si possono azzerare. È un limite inevitabile in una UI web
(Tauri non ha un campo password nativo). Va documentato, e si riduce al minimo: niente segreti negli store
(Zustand/Query), campo svuotato subito dopo l'invio, nessun log. Nella direzione opposta il frontend **non riceve
mai** segreti.

### Eventi e canali
- **Eventi** (`use tauri::Emitter; app.emit("document-changed", payload)`, `emit_to`, `emit_filter`; in JS
  `listen<T>()` da `@tauri-apps/api/event`): la doc dice che sono pensati per "small amounts of data" e che "it might
  not be suitable to sending a large amount of data" (valutano JS direttamente, il payload è JSON). Vanno bene per
  notifiche rare: documento modificato all'esterno, vault bloccato, fine sessione.
- **Canali** (`tauri::ipc::Channel<T>`): "designed to be fast and deliver ordered data". Vanno usati per terminale,
  avanzamento dell'import e in futuro RDP. Per i byte si usa `Channel<InvokeResponseBody>` con
  `InvokeResponseBody::Raw(Vec<u8>)`: in JS arrivano come `ArrayBuffer`. **`Channel<Vec<u8>>` invece produce un
  array JSON di numeri** (docs.rs, `Channel`).
- In JS l'`onmessage` va assegnato **prima** di passare il canale al comando: i messaggi arrivati prima non vengono
  riprodotti.

**Fonti:** https://v2.tauri.app/develop/state-management/ · https://v2.tauri.app/develop/calling-frontend/ ·
https://docs.rs/tauri/2.12.5/tauri/ipc/struct.Channel.html · https://v2.tauri.app/reference/javascript/api/namespacecore/

**Rischi:** un guard di `Mutex` tenuto attraverso un `.await` (deadlock o `!Send`); se un comando va in panic
con il lock preso, il `Mutex` resta avvelenato (già gestito in `LazyVault`, va gestito anche per la sessione).

---

## 4. Terminale SSH integrato

### PTY: **`portable-pty` 0.9.0** (raccomandato) e alternative
| Crate | Versione | Note |
|---|---|---|
| `portable-pty` | **0.9.0** (2025-02-11, unica stabile) | parte di wezterm, ~16,6M download recenti; dipendenze datate (`nix ^0.28`, `bitflags ^1.3`, `lazy_static`, `anyhow`) ma API stabile e diffusissima (anche nei terminali Tauri) |
| `pty-process` | 0.5.4 (2026-10-19) | solo Unix, wrapper su `std`/`tokio::process::Command`, async con la feature `async`; **la build di docs.rs della 0.5.4 è fallita** |
| fatto in casa (`openpty` + `pre_exec`) | — | richiede `unsafe` (vietato dal workspace) |

API verificata sul sorgente (`wezterm/pty/src/lib.rs`, `cmdbuilder.rs`):
- `native_pty_system().openpty(PtySize { rows, cols, pixel_width, pixel_height }) -> PtyPair { master, slave }`;
- `MasterPty: Send` con `resize(PtySize)`, `try_clone_reader() -> Box<dyn Read + Send>`,
  `take_writer() -> Box<dyn Write + Send>`, `process_group_leader()`;
- `SlavePty::spawn_command(CommandBuilder) -> Box<dyn Child + Send + Sync>`; `Child`: `try_wait`, `wait`,
  `process_id`; `ChildKiller::kill`, `clone_killer() -> Box<dyn ChildKiller + Send + Sync>`;
  `ExitStatus::{exit_code() -> u32, success(), signal() -> Option<&str>}`;
- `CommandBuilder::new` **copia l'ambiente del processo padre** (`get_base_env`), e allo spawn fa `env_clear()` e
  applica la propria copia. Quindi `env_remove` funziona davvero. Il `TERM` **non** viene impostato.

> Nota: la API l'ho letta sul ramo `main` di wezterm; la 0.9.0 pubblicata dovrebbe coincidere **[da verificare]**
> con `cargo doc` dopo l'aggiunta.

### Collegamento con `prepare` / `PreparedLaunch` (verificato nel codice)
`portolano_connectors::prepare(plan, vault, config) -> PreparedLaunch` espone `program()`, `args()`, `env()`
(comprese le variabili askpass `SSH_ASKPASS`, `SSH_ASKPASS_REQUIRE=force`, `PORTOLANO_ASKPASS=1`,
`PORTOLANO_ASKPASS_SOCK`, `PORTOLANO_ASKPASS_TOKEN`), `stdin_secret()` e `wait()`. Il drop ferma il server askpass e
azzera i segreti. Il doc di `env()` dice: "A caller that starts the program itself must also remove
`MASTER_PASSWORD_ENV`". `PreparedLaunch::command()`, che rimuove le 4 variabili, è privato.

**Raccomandazione:** esporre in `portolano-connectors` una costante pubblica `CHILD_ENV_REMOVE: [&str; 4]` (o un
helper `apply_child_env`) per non duplicare l'elenco nell'app.

```rust
fn spawn_ssh(prepared: &PreparedLaunch, size: PtySize)
    -> anyhow::Result<(Box<dyn MasterPty + Send>, Box<dyn Child + Send + Sync>)>
{
    // Il terminale integrato serve solo per piani SSH che finiscono con Run (niente --new-window, niente FreeRDP).
    anyhow::ensure!(prepared.stdin_secret().is_none(), "piano non adatto al terminale integrato");
    let pair = portable_pty::native_pty_system().openpty(size)?;
    let mut cmd = portable_pty::CommandBuilder::new(prepared.program());
    cmd.args(prepared.args());
    for name in CHILD_ENV_REMOVE { cmd.env_remove(name); }   // DEC-023.1
    for (key, value) in prepared.env() { cmd.env(key, value); }
    cmd.env("TERM", "xterm-256color");
    let child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);                                         // altrimenti la lettura non vede mai EOF
    Ok((pair.master, child))
}
```
La `TerminalSession` **possiede il `PreparedLaunch`** (il server askpass deve vivere quanto `ssh`, come già fa
`launch()`), il `MasterPty`, il writer e il killer. Thread previsti:
- **lettore:** `read` in un buffer da 16–64 KiB, poi `on_data.send(InvokeResponseBody::Raw(buf[..n].to_vec()))`;
  si ferma su EOF o errore;
- **attesa:** `child.wait()`, poi `on_exit.send(TerminalEvent::Exited { code })` (un canale JSON separato, oppure un
  enum taggato) e drop del `PreparedLaunch` (socket e segreti eliminati). `ssh` esce con 255 per gli errori di
  connessione: la UI può mostrarlo come "connessione fallita".

Comandi: `terminal_open(node_id, cols, rows, on_data: Channel<InvokeResponseBody>, on_exit: Channel<TerminalEvent>) -> TerminalId`,
`terminal_write(id, data: String)` (oppure byte grezzi: `invoke` accetta `ArrayBuffer`/`Uint8Array`),
`terminal_resize(id, cols, rows)` → `master.resize(PtySize{..})`, `terminal_close(id)` → `killer.kill()`, poi
l'evento di uscita arriva dal thread di attesa.

**Fallback dell'askpass:** il client askpass, quando il server risponde `fallback` (conferma della host key, prompt
non riconosciuti), apre `/dev/tty`. Lanciato da `ssh` dentro la nostra PTY, eredita la PTY come terminale di
controllo, quindi **il prompt compare in xterm.js**. Il flusso della CLI è già pensato così, ma va provato dal
vivo **[da verificare]**.

**`Environment`:** `Environment::discover` usa `current_exe()` come `self_exe`, che nell'app sarebbe l'eseguibile
Tauri. L'app deve sovrascrivere `env.self_exe` (il campo è pubblico) con il percorso del sidecar (§5). Il `PATH` di
un'app aperta dal Finder è minimo (`/usr/bin:/bin:/usr/sbin:/sbin`): `ssh` e `ssh-add` stanno in `/usr/bin`, e
FreeRDP ha già le directory extra (`FREERDP_EXTRA_DIRS`). `SSH_AUTH_SOCK` lo fornisce launchd anche alle app GUI.

### Frontend: xterm.js
| Pacchetto | Versione |
|---|---|
| `@xterm/xterm` | **6.0.0** (2025-12-22; beta 6.1.0) |
| `@xterm/addon-fit` | **0.11.0** |
| `@xterm/addon-web-links` | **0.12.0** |
| `@xterm/addon-webgl` | 0.19.0 (facoltativo) |
| `@xterm/addon-unicode11` / `addon-search` | 0.9.0 / 0.16.0 (facoltativi) |

Novità della 6.0: **il renderer canvas è stato rimosso** (si usa il DOM, il predefinito, oppure WebGL); sono state
eliminate le opzioni deprecate `windowsMode` e `fastScrollModifier`; la scrollbar è stata rifatta; c'è il supporto
ESM e OSC 52.
```ts
const term = new Terminal({ fontFamily: "IBM Plex Mono", cursorBlink: true });
const fit = new FitAddon();
term.loadAddon(fit);
term.loadAddon(new WebLinksAddon((_e, uri) => void openExternal(uri)));   // via plugin-opener, solo http(s)
term.open(el);
fit.fit();

const onData = new Channel<ArrayBuffer>();
onData.onmessage = (buf) => term.write(new Uint8Array(buf));
const onExit = new Channel<TerminalEvent>();
onExit.onmessage = (ev) => markEnded(ev.code);
const id = await commands.terminalOpen(nodeId, term.cols, term.rows, onData, onExit);

term.onData((d) => void commands.terminalWrite(id, d));
term.onResize(({ cols, rows }) => void commands.terminalResize(id, cols, rows));
new ResizeObserver(() => fit.fit()).observe(el);
```
Sicurezza del terminale: il server remoto può inviare sequenze OSC. Il titolo (`onTitleChange`) va mostrato solo come
testo; i link solo `http`/`https` e aperti tramite `@tauri-apps/plugin-opener`; il clipboard OSC 52 va lasciato
spento (non caricare l'addon clipboard), o almeno solo in scrittura e con conferma.

**Fonti:** https://docs.rs/portable-pty/0.9.0/portable_pty/ · https://github.com/wezterm/wezterm/tree/main/pty/src ·
https://docs.rs/pty-process/0.5.4/pty_process/ · https://github.com/xtermjs/xterm.js/releases/tag/6.0.0

**Rischi:** `portable-pty` non esce in versione nuova da febbraio 2025 (ma wezterm è attivo); la contropressione
(xterm `write` con callback) per output molto voluminosi; differenze fra la API letta su `main` e la 0.9.0.

---

## 5. Askpass dentro l'app

### Fatti verificati
- `bundle.externalBin: ["binaries/portolano"]` richiede il file `binaries/portolano-aarch64-apple-darwin`
  (`rustc --print host-tuple`).
- **Nel bundle** il sidecar va in `Contents/MacOS/` (`tauri-bundler/src/bundle/macos/app.rs`:
  `bin_dir = bundle_directory.join("MacOS")`) con il **suffisso della triple rimosso**
  (`settings.rs::copy_binaries`: `.replace(&format!("-{}", self.target), "")`). Viene **firmato** con l'app, "inside
  out" (sidecar prima del bundle).
- **Durante `cargo build`**, `tauri-build` copia i sidecar in `target/<profilo>/` togliendo la triple, e **fallisce la
  build se il file manca** (`tauri-build/src/lib.rs`).
- Il plugin shell risolve il sidecar con `current_exe().parent().join(nome)`
  (`plugins/shell/src/process/mod.rs::relative_command_path`). **Per ottenere solo il percorso il plugin shell non
  serve**: basta lo stesso calcolo. Il plugin servirebbe solo a *lanciare* il sidecar dal frontend, e noi non lo
  vogliamo.

### Opzione (a): CLI `portolano` come sidecar
Pro:
- un solo eseguibile gestisce già `PORTOLANO_ASKPASS=1` prima di clap (DEC-009); nessun nuovo codice;
- **serve comunque** per "Apri in nuova finestra" (`OpenInTerminal` rilancia `<self_exe> … connect @<id>`), che
  richiede la CLI completa, come annotato in `progress.md` ("`self_exe` … = binario CLI incluso nell'app");
- regala all'utente una CLI coerente con l'app (`/Applications/Portolano.app/Contents/MacOS/portolano`, che si può
  collegare con un symlink);
- il processo askpass resta piccolo (7 MB, nessun framework WebKit/AppKit caricato).

Contro:
- circa +7 MB al bundle;
- c'è un secondo binario da firmare (il bundler lo fa da solo);
- **conflitto nel workspace:** `tauri-build` copia `binaries/portolano-<triple>` in `target/<profilo>/portolano`,
  cioè **sopra il binario della CLI compilato dallo stesso workspace**; e se il file manca, la build del crate
  dell'app (quindi `cargo clippy --workspace`) fallisce;
- per il Keychain, la CLI è un binario diverso dall'app: quando legge i segreti (nuova finestra), macOS chiede
  l'autorizzazione separatamente (vedi §12).

**Soluzione proposta al conflitto** (da provare nel primo WP):
1. **Non** mettere `externalBin` nel `tauri.conf.json` di base, ma in un file `tauri.bundle.json` passato solo a
   `tauri build --config tauri.bundle.json`, preceduto da uno script che compila `portolano-cli --release` e lo
   copia in `binaries/portolano-$(rustc --print host-tuple)`. Così `cargo build/clippy/test --workspace` restano
   indipendenti. **[da verificare: che `tauri-build` legga la config passata con `--config`, tramite la variabile
   `TAURI_CONFIG`]**
2. A runtime, sia in sviluppo sia nel bundle, si usa **la stessa regola**:
   `current_exe()?.parent()?.join("portolano")`. In sviluppo l'eseguibile dell'app è in `target/debug/` accanto alla
   CLI del workspace, quindi basta un `cargo build -p portolano-cli` nel `beforeDevCommand`. Prima di usarlo si
   controlla che il file esista e sia eseguibile; altrimenti si mostra un errore chiaro ("binario CLI mancante").

### Opzione (b): l'eseguibile dell'app gestisce l'askpass
`main.rs` controlla `is_client_invocation()` prima di `tauri::Builder` e chiama `run_client(prompt)`.

Pro: nessun sidecar né conflitto di build; un binario in meno da firmare.

Contro: **non risolve la "nuova finestra"**, perché l'app dovrebbe imitare anche `connect`, duplicando la CLI. Inoltre
`ssh` lancerebbe come askpass l'eseguibile di un bundle GUI: funziona se si esce prima di inizializzare AppKit, ma
carica i framework a ogni prompt e mescola i ruoli.

### Raccomandazione: **(a) sidecar CLI**, con `externalBin` solo nella config di bundle e il percorso calcolato
come `current_exe().parent().join("portolano")`. Si tiene (b) come ripiego, solo se la nuova finestra venisse
tolta dall'app.

**Fonti:** https://v2.tauri.app/develop/sidecar/ ·
https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/macos/app.rs ·
https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/settings.rs ·
https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-build/src/lib.rs ·
https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/shell/src/process/mod.rs

---

## 6. Sicurezza Tauri v2

### Capabilities e permessi
- I file JSON/TOML in `src-tauri/capabilities/` sono **tutti attivi per default**; se `app.security.capabilities`
  li elenca, valgono solo quelli elencati. I campi sono `identifier`, `windows` (per **label**, non per titolo),
  `permissions`, `platforms`; più capability sulla stessa finestra **si sommano**.
- **Comandi dell'app:** per default *tutti* i comandi registrati con `invoke_handler` sono permessi a ogni
  finestra. Per restringerli si usa `AppManifest::commands` in `build.rs`
  (`tauri_build::try_build(Attributes::new().app_manifest(AppManifest::new().commands(&[...])))`): genera i permessi
  `allow-<comando>` da elencare nella capability. Raccomando di farlo: una sola finestra oggi, ma in futuro una
  finestra separata (per esempio un pannello RDP) potrà avere solo i comandi che le servono.
- Capability proposta:
```json
{
  "identifier": "main",
  "windows": ["main"],
  "permissions": [
    "core:event:default",
    "core:window:default",
    "core:window:allow-start-dragging",
    "dialog:allow-open", "dialog:allow-save", "dialog:allow-message", "dialog:allow-ask",
    "opener:allow-open-url",
    "allow-open-document", "allow-tree", "allow-node-details", "allow-terminal-open", "…"
  ]
}
```
  (`core:window:allow-start-dragging` serve per `data-tauri-drag-region`. Per `opener:allow-open-url` si può limitare
  lo scope a `https://*` e `http://*`. I nomi esatti dei permessi dei plugin vanno presi dai file `permissions/`
  generati **[da verificare]**.)
- **Da non includere:** `tauri-plugin-shell` (spawn di processi dal frontend), `tauri-plugin-fs`,
  `tauri-plugin-http`. Nessun `withGlobalTauri`, nessuna capability `remote`.
- **I dialoghi di file conviene aprirli da Rust** (`app.dialog().file().blocking_pick_file()` dentro un comando
  `open_document_dialog`). Così il frontend non sceglie *percorsi arbitrari* da passare a `open_document(path)`, e un
  frontend compromesso non può far aprire o scrivere file a piacere. Se il dialogo resta in JS, il comando deve
  comunque validare il percorso (estensione, file regolare).

### CSP per un'app solo locale (proposta)
```json
"csp": {
  "default-src": "'self'",
  "script-src": "'self'",
  "style-src": "'self' 'unsafe-inline'",
  "img-src": "'self' data: blob: portolano-icon:",
  "font-src": "'self'",
  "connect-src": "ipc: http://ipc.localhost",
  "object-src": "'none'",
  "base-uri": "'none'",
  "form-action": "'none'",
  "frame-src": "'none'"
},
"devCsp": { "…": "come sopra + connect-src ws://localhost:5173 http://localhost:5173 per HMR" }
```
- Tauri aggiunge da solo hash e nonce degli asset del bundle ("Local scripts are hashed, styles and external scripts
  are referenced using a cryptographic nonce"); `connect-src ipc: http://ipc.localhost` è quello dell'esempio
  ufficiale.
- `style-src 'unsafe-inline'` serve con buona probabilità: xterm.js (renderer DOM) e alcune librerie inseriscono
  elementi `<style>` a runtime. Gli `style={{…}}` di React passano dal CSSOM e non sono bloccati. È un rischio
  basso, perché riguarda solo stili e non script. Si può provare a farne a meno e misurare. **[da verificare]**
- Il mock carica **Google Fonts** e React da **unpkg**: nell'app si includono con `@fontsource/hanken-grotesk` e
  `@fontsource/ibm-plex-mono` (5.3.0) e dipendenze npm. Nessun contenuto remoto.
- Se un giorno IronRDP girasse in WASM nella webview, servirebbe `script-src 'wasm-unsafe-eval'` (doc CSP). È un
  motivo in più per preferire la variante nativa (§11).
- `freezePrototype: true` (prototype pollution) e `assetProtocol.enable: false`.

### Isolation pattern
Intercetta ogni messaggio IPC in un iframe isolato, cifrato con AES-GCM e con una chiave per sessione; la doc lo
consiglia "whenever possible", con un costo che "most applications should not notice". Protegge dalle dipendenze
npm compromesse. **Raccomandazione:** non è indispensabile al primo rilascio (CSP stretta, nessun contenuto
remoto, comandi ristretti con AppManifest e validazione degli input in Rust coprono il grosso). Lo metterei come
miglioramento con un validatore minimo (allowlist dei nomi di comando), da misurare con lo streaming del terminale.
Feature cargo `isolation`.

### Icone dell'IconStore
Le icone stanno in `<dir del documento>/<stem>.icons/<sha256>.<ext>` (png, jpeg, gif, tiff, svg, pdf).
`IconStore::path_for(&Icon) -> Option<PathBuf>` esiste già. Le opzioni:
1. **Asset protocol** (`convertFileSrc`): richiede la feature `protocol-asset`, `assetProtocol.enable` e uno scope
   *dinamico*, perché il documento può stare ovunque (`asset_protocol_scope().allow_directory(...)`). È la superficie
   più ampia (lettura di file per percorso).
2. **Base64 via comando:** semplice, ma ogni icona passa in JSON, non c'è cache HTTP e l'`<img src="data:…">` va
   ricostruito a ogni render.
3. **Raccomandato: schema URI personalizzato** `register_asynchronous_uri_scheme_protocol("portolano-icon", …)`. Su
   macOS l'URL è `portolano-icon://localhost/<sha256>.<ext>` (docs.rs `Builder`). Il gestore:
   - accetta solo nomi `^[0-9a-f]{64}\.(png|jpg|gif|tiff|svg|pdf)$`;
   - legge **solo** dalla directory icone del documento aperto;
   - imposta `Content-Type` dall'estensione, `X-Content-Type-Options: nosniff` e, per gli SVG,
     `Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'`;
   - nella CSP dell'app: `img-src … portolano-icon:`.

   Nel frontend l'icona si mostra **solo** come `<img src>`, mai inline, mai `dangerouslySetInnerHTML`, mai
   `<object>` o `<iframe>` (DEC-024). Un SVG dentro `<img>` non esegue script.
   **[da verificare]:** che WKWebView mostri dentro `<img>` anche TIFF (Safari lo supporta) e PDF (prima pagina);
   altrimenti si converte o si mostra un segnaposto.

Inoltre: i nomi, gli host e gli utenti del documento arrivano da fonti non fidate. React fa l'escape del testo, ma
i caratteri di controllo e bidi vanno resi visibili come nella CLI (DEC-024.3, DEC-026.4) con un helper comune.

**Fonti:** https://v2.tauri.app/security/capabilities/ · https://v2.tauri.app/security/csp/ ·
https://v2.tauri.app/concept/inter-process-communication/isolation/ ·
https://docs.rs/tauri/2.12.5/tauri/struct.Builder.html · https://v2.tauri.app/plugin/dialog/ ·
https://v2.tauri.app/learn/window-customization/

---

## 7. Finestra macOS

- **Titlebar:** `titleBarStyle: "Overlay"` + `hiddenTitle: true` + `trafficLightPosition`. La barra la disegna la UI,
  con `data-tauri-drag-region` (vale solo sull'elemento su cui è scritto, non sui figli) e il permesso
  `core:window:allow-start-dragging`. La doc avverte che una titlebar personalizzata su macOS "will also lose some
  features provided by the system". `Overlay` con i semafori nativi ne conserva la maggior parte (doppio clic per
  zoom: **[da verificare]**).
- **Vibrancy / vetro: integrati in Tauri**, senza crate esterno. `windowEffects.effects` accetta i materiali
  `NSVisualEffectView` (`sidebar`, `hudWindow`, `underWindowBackground`, `contentBackground`, `windowBackground`, …)
  e, **su macOS 26+, `liquidGlassRegular` / `liquidGlassClear`**. Si possono combinare uno stile Liquid Glass e un
  materiale, e su macOS ≤ 15 si ripiega sul materiale. Ci sono anche `state`, `radius`, `color` (solo Liquid Glass) e
  `interactive` (macOS 27+). Richiede `transparent: true`; `macOSPrivateApi` non serve più (no-op da 2.12.1).
  Il crate `window-vibrancy` 0.8.1 esiste (2026-09-22), ma **non serve**.
- **Impatto sul mock:** il mock usa molto `backdrop-filter` (App: 26× `blur(18px)`, 18× `blur(6px) saturate(140%)`;
  Brand fino a `blur(30px) saturate(150%)`).
  - WebKit supporta `backdrop-filter` senza prefisso da Safari 18; con macOS 27 non ci sono problemi di supporto.
  - **Un `backdrop-filter` CSS sfoca solo il contenuto della webview, non il desktop dietro la finestra.** Per il
    "vetro" sopra il desktop serve l'effetto nativo (`windowEffects`) con sfondo `html, body { background:
    transparent }`. Il glassmorphism CSS va bene per i pannelli sopra lo sfondo dell'app.
  - **Prestazioni:** ogni elemento con `backdrop-filter` è un layer composito che va ricalcolato quando cambia ciò che
    sta sotto. **Uno sfondo animato sotto molti pannelli sfocati** costringe a ricalcolare la sfocatura a ogni frame
    (GPU, batteria, scatti con xterm aperto).
  - Indicazioni: animazione lenta, solo `transform`/`opacity`, sospesa con la finestra non attiva
    (`visibilitychange`, evento focus di Tauri) e con `prefers-reduced-motion`. Pochi livelli di blur sovrapposti e
    raggi moderati (≤ 20px). Niente blur sul pannello del terminale (testo che cambia continuamente). Va misurato con
    Web Inspector di Safari (in sviluppo Tauri apre i devtools).
  - **Raccomandazione:** materiale o Liquid Glass nativo per lo sfondo della finestra o della sidebar, blur CSS
    leggero e limitato a dialoghi e popover, sfondo animato facoltativo e spegnibile (preferenza di aspetto).

**Fonti:** https://v2.tauri.app/reference/config/ (WindowConfig, WindowEffectsConfig, macOSPrivateApi) ·
https://v2.tauri.app/learn/window-customization/ · https://crates.io/crates/window-vibrancy ·
https://developer.mozilla.org/docs/Web/CSS/backdrop-filter

---

## 8. Frontend

### Versioni (npm `latest`, verificate)
| Pacchetto | Versione | Note |
|---|---|---|
| `react`, `react-dom` | **19.3.1** | |
| `@types/react`, `@types/react-dom` | 19.3.0 | |
| `vite` | **8.3.8** | Node `^20.19 \|\| >=22.12` |
| `@vitejs/plugin-react` | **6.3.0** | peer `vite ^8`; senza Babel per default (oxc) |
| `typescript` | **6.0.3** (non la 7.0.2) | vedi sotto |
| `zustand` | 5.1.0 | |
| `@tanstack/react-query` | 5.107.1 | |
| `@headless-tree/core`, `@headless-tree/react` | 1.8.0 (2026-10-25) | albero con DnD, a11y, typeahead, rinomina; virtualizzabile |
| `@tanstack/react-virtual` | 3.14.26 | |
| `react-hook-form` | 7.92.0 | |
| `zod` | 4.6.7 | |
| `@hookform/resolvers` | 7.0.5 | peer `zod ^3.25 \|\| ^4` |
| `eslint` | 10.14.0 | |
| `@eslint/js` | 10.0.1 | |
| `typescript-eslint` | 8.72.2 | peer **`typescript >=4.8.4 <6.1.0`** |
| `eslint-plugin-react-hooks` | 7.1.1 | peer ESLint fino a 10 |
| `eslint-plugin-react-refresh` | 0.5.7 | |
| `prettier` | 3.10.0 | |
| `@biomejs/biome` | 2.6.6 | alternativa, non raccomandata qui |

**TypeScript 7 (porting nativo) è `latest` su npm dal 2026-07-08, ma `typescript-eslint` 8.72.2 supporta solo
`<6.1.0`** (anche la pagina "dependency-versions" dice `>=4.8.4 <6.1.0` e non parla di TS 7). Va **fissato
`typescript@~6.0.3`**. Si può riconsiderare quando typescript-eslint supporterà la 7.

### Raccomandazioni
- **Stato:** **TanStack Query** per i dati del backend (albero, dettagli, validazione, config), con
  `invalidateQueries` dopo le mutazioni e su eventi come `document-changed` o `modified-externally`; **Zustand** per lo
  stato della UI (selezione, tab e sessioni aperte, pannelli, vault bloccato o sbloccato). Combinazione semplice e
  testabile, senza Redux. Nessun segreto in nessuno dei due.
- **Albero:** **Headless Tree** (successore ufficiale di react-complex-tree: "official successor for React Complex
  Tree"; MIT; DnD ordinato, accessibilità ARIA, typeahead, rinomina, dati asincroni, compatibile con le librerie di
  virtualizzazione) con **`@tanstack/react-virtual`** per alberi grandi. Alternative scartate:
  - `react-arborist` 4.0.1: peer `react-dnd ^16` (stack DnD vecchio);
  - `@dnd-kit/core` 6.3.1: ferma a dicembre 2024; la nuova `@dnd-kit/react` è 0.5.3 (pre-1.0);
  - per riordinare le tab, se servirà, si valuterà dnd-kit; per l'albero non serve.

  Rischio: progetto con un solo manutentore principale (≈910 stelle).
- **Form:** react-hook-form + zod (+ `@hookform/resolvers`). La validazione definitiva resta in Rust (ops/protocolli);
  quella del form serve solo all'esperienza d'uso, e l'errore `Invalid` del backend va mostrato sul campo.
- **Lint e formattazione:** **ESLint 10 flat config + typescript-eslint (preset `recommendedTypeChecked`) +
  react-hooks + react-refresh**, più **Prettier**. Le regole che usano i tipi (`no-floating-promises`,
  `no-misused-promises`) sono preziose con `invoke` asincrono e Biome non le copre allo stesso modo. Quality gate:
  `tsc --noEmit`, `eslint . --max-warnings 0`, `prettier --check .`.
- **Configurazione Vite** (dalla doc Tauri): `clearScreen: false`, `server.port` uguale a `devUrl` e
  `strictPort: true`, `envPrefix: ['VITE_', 'TAURI_ENV_*']`, `watch.ignored: ['**/src-tauri/**']` (qui
  `../crates/**`), `build.target` WebKit (la doc usa `safari13`; per macOS 14+ si può alzare, per esempio `safari17`),
  `minify: !process.env.TAURI_ENV_DEBUG`, `sourcemap: !!process.env.TAURI_ENV_DEBUG`.

**Fonti:** registro npm (`npm view`, 2026-11-11) · https://typescript-eslint.io/users/dependency-versions/ ·
https://headless-tree.lukasbach.com/ · https://github.com/lukasbach/headless-tree ·
https://v2.tauri.app/start/frontend/vite/

---

## 9. Test

| Pacchetto | Versione | Note |
|---|---|---|
| `vitest` | **5.1.0** | Node `^22.12 \|\| ^24 \|\| >=26`; peer `vite ^6.4\|\|^7\|\|^8` |
| `jsdom` | **31.0.0** | Node `^22.22.2 \|\| ^24.15 \|\| >=26` |
| `@testing-library/react` | 16.3.3 | peer `@testing-library/dom ^10` |
| `@testing-library/dom` | 10.4.2 | |
| `@testing-library/user-event` | 14.6.9 | |
| `@testing-library/jest-dom` | 7.0.1 | |
| `@playwright/test` | 1.65.0 | |
| `@wdio/tauri-service` / `@wdio/cli` / `webdriverio` | 1.5.1 / 10.2.0 / 10.2.0 | facoltativi |
| crate `tauri-plugin-wdio-webdriver`, `tauri-plugin-wdio` | 1.5.1 | facoltativi, dietro una feature |

1. **Unit e component test del frontend:** Vitest + jsdom + Testing Library, con `mockIPC` da
   `@tauri-apps/api/mocks`: `mockIPC((cmd, args) => …)`, `mockWindows('main')`, `clearMocks()` dopo ogni test,
   `mockIPC(handler, { shouldMockEvents: true })` per gli eventi (`emitTo`/`emit_filter` non sono supportati). La doc
   aggiunge un polyfill di `window.crypto` con `randomFillSync` per jsdom (con jsdom 31 e Node 24 potrebbe non
   servire **[da verificare]**). **La simulazione dei `Channel` non è documentata**: conviene isolare il trasporto
   del terminale dietro un'interfaccia (`TerminalTransport`) e usarne una finta nei test, invece di simulare l'IPC.
   Se i bindings sono di tauri-specta, si può anche simulare il modulo `bindings.ts` con `vi.mock`.
2. **Comandi Rust:** la logica si testa *fuori* da Tauri (funzioni pure che ricevono `&AppState` o i suoi pezzi; ops
   ha già i suoi test). Per l'IPC vero c'è il modulo `tauri::test` (feature cargo **`test`**, "This module is
   unstable"): `mock_builder()`, `mock_context(noop_assets())`, `get_ipc_response(&webview, InvokeRequest{…})`,
   `INVOKE_KEY`. Serve a verificare la serializzazione di `IpcError` e i permessi.
3. **End-to-end sull'app vera in macOS:** **`tauri-driver` da solo non supporta macOS** ("only Windows and Linux are
   supported on desktop, as macOS has no WKWebView driver tool available"). La doc ufficiale oggi indica
   **WebdriverIO con `@wdio/tauri-service`** e un **server WebDriver incorporato nell'app**
   (`tauri-plugin-wdio-webdriver`): "this is how macOS is supported". In alternativa c'è il fork CrabNebula, che
   **su macOS richiede una chiave API a pagamento**. Il plugin incorporato apre un server WebDriver *dentro* l'app,
   quindi va compilato **solo** con una feature cargo `e2e`, mai nel bundle distribuito.
4. **Alternativa realistica e consigliata come primo passo:** **Playwright sul solo frontend** (Vite in modalità
   "mock", con `mockIPC` o un finto backend in memoria), usando il **progetto WebKit** di Playwright, il motore più
   vicino a WKWebView. Il flusso reale con `ssh`, il Keychain e la PTY resta nei test Rust e nella verifica manuale.
   Il WebdriverIO incorporato si aggiunge dopo, se servono test sul bundle vero.

**Fonti:** https://v2.tauri.app/develop/tests/mocking/ · https://v2.tauri.app/develop/tests/webdriver/ ·
https://docs.rs/tauri/2.12.5/tauri/test/index.html · https://crates.io/crates/tauri-plugin-wdio-webdriver

**Rischi:** l'IPC simulato non vede gli errori di serializzazione reali (vanno coperti dai test con `tauri::test`); il
plugin WebDriver incorporato non deve finire nel bundle di release.

---

## 10. Documento modificato all'esterno

- `notify` **8.2.2** (2026-11-08, MSRV 1.88; esiste la 9.0.0-rc.8, da non usare) + `notify-debouncer-full` **0.7.0**
  (dipende da `notify ^8.2.0`, quindi le due versioni sono compatibili). Su macOS il backend predefinito è FSEvents
  (feature `macos_fsevent`); kqueue è un'alternativa.
- Lo dice la doc: gli editor "Some truncate the file on save, some create a new one and replace the old one"; i
  filesystem di rete "may not emit any events" (`PollWatcher`). Anche il nostro `Store::save` scrive in un file
  temporaneo e poi fa la **rename** (e scrive `<file>.bak`). **Quindi si osserva la directory padre
  (`RecursiveMode::NonRecursive`)** e si filtrano gli eventi sul percorso del documento (canonicalizzato: lo `Store`
  risolve i symlink); osservare il solo file perderebbe la sostituzione via rename.
- **Per distinguere i propri salvataggi** non bastano gli eventi: dopo il debounce (300–500 ms) si confronta
  l'impronta. Lo `Store` la tiene già (SHA-256, DEC-016.1), ma non la espone. Serve una piccola aggiunta al core:
  `Store::changed_on_disk(&self) -> Result<bool>` (rilegge sotto lock condiviso e confronta con l'impronta
  ricordata). Se il file è cambiato si emette `document-modified-externally`; la UI propone "Ricarica" e, se ci sono
  modifiche nell'editor, le tiene come patch (`progress.md`). `save` resta protetto in ogni caso da
  `ModifiedExternally`.
- Il watcher si ricrea a ogni cambio di documento e si ferma alla chiusura. Si osservano anche la cancellazione e lo
  spostamento del documento, per mostrare lo stato `DocumentMissing`.

**Fonti:** https://docs.rs/notify/8.2.2/notify/ · https://docs.rs/notify-debouncer-full/0.7.0/notify_debouncer_full/

---

## 11. RDP integrato futuro (IronRDP)

**Stato attuale (crates.io e README):**
- crate meta `ironrdp` **0.17.0** (2026-07-10, MSRV 1.89);
- crate singoli più recenti: `ironrdp-connector` 0.12.0, `ironrdp-session` 0.13.0, `ironrdp-tls` 0.3.0,
  `ironrdp-cliprdr` 0.9.0, `ironrdp-blocking` 0.12.0, `ironrdp-rdcleanpath` 0.3.1, tutti di ottobre 2026 con MSRV
  1.94 (Rust 1.99 va bene);
- **`ironrdp-client` 0.1.0** (2026-07-10): "Portable RDP client engine without GPU acceleration". Espone `Config`, il
  runtime `RdpClient`, eventi di input e output, il trasporto WebSocket e il driver di sessione; l'output arriva su
  "a bounded `tokio::sync::mpsc::Sender<RdpOutputEvent>`", e la libreria non dipende da winit, cioè è pensata per
  essere incorporata. La build di docs.rs è fallita, quindi l'API non si legge online.
- `ironrdp-viewer` 0.1.0 è il client nativo con finestra;
- il web component `@devolutions/iron-remote-desktop` 0.11.0 e `@devolutions/iron-remote-desktop-rdp` 0.7.0 (npm,
  2026-05) usano binding WASM (`ironrdp-web`, non pubblicato su crates.io: compilato con `wasm-pack`).

Funzioni dal README: TLS 1.2/1.3, NLA (CredSSP con NTLM/Kerberos), **RD Gateway (MS-TSGU)**, RDCleanPath, parsing dei
file `.rdp`; decodifica bitmap/RLE/RemoteFX; canali per clipboard, audio, RDPDR, display control. **H.264 non è
elencato** fra le decodifiche del client. Licenza MIT/Apache-2.0.

**Due modi per integrarlo in una tab:**
1. **WASM nella webview** (web component di Devolutions). Il browser non apre TCP, quindi serve un proxy WebSocket →
   TCP con RDCleanPath. Nell'app sarebbe un server locale in Rust (`ironrdp-rdcleanpath`) in ascolto su localhost. In
   più la CSP richiede `'wasm-unsafe-eval'` e `connect-src ws://127.0.0.1:*`, TLS e CredSSP girano nella webview, e
   **la password dovrebbe arrivare al frontend** ⇒ **viola il vincolo "il frontend non riceve segreti"**. Scartato.
2. **Nativo in Rust (raccomandato):** `ironrdp-client` / `ironrdp-session` sul runtime tokio di Tauri. I segreti
   arrivano dal vault come per FreeRDP, senza passare dalla webview. I frame (rettangoli modificati in RGBA o BGRA)
   vanno al frontend su un `Channel<InvokeResponseBody::Raw>` con un piccolo header (x, y, w, h); il frontend li
   disegna su `<canvas>` con `putImageData`, o con WebGL se serve. Input di mouse e tastiera tramite comandi (o un
   canale inverso, cioè `invoke` con payload binario). Il ridimensionamento passa da display control.

**Da predisporre ora** (DEC-027: "la UI prevede fin da ora lo spazio per il pannello RDP"):
- **modello di "sessione" indipendente dal protocollo** nel frontend (Zustand):
  `Session { id, nodeId, protocol, kind: "terminal" | "rdp" | "external", state: connecting|connected|ended|error, exitCode? }`,
  e un registro `sessionViews[kind] → componente`. La tab non sa nulla di xterm né di canvas;
- **backend:** un `SessionManager` generico (id, chiusura, evento di fine) con un trait per i tipi di sessione;
  `TerminalSession` è il primo. Lo stesso schema di trasporto: un canale binario per l'output, un canale JSON per gli
  eventi di stato, comandi `session_input` / `session_resize` / `session_close` validi per tutti;
- **contratto di trasporto binario** fin da subito (`InvokeResponseBody::Raw` più un header versionato), così il
  canale RDP non cambia l'architettura;
- i piani di lancio restano nei connettori: per RDP ci sarà una "preparazione integrata" parallela a `prepare` (il
  protocollo dice se sa fare sessioni integrate), senza toccare il core (SPEC §1.5);
- CSP e capability già pronte per una seconda vista, **senza** `wasm-unsafe-eval`.

**Fonti:** https://github.com/Devolutions/IronRDP · https://docs.rs/ironrdp-client/0.1.0/ironrdp_client/ ·
https://github.com/Devolutions/IronRDP/tree/master/crates/ironrdp-web ·
https://www.npmjs.com/package/@devolutions/iron-remote-desktop

**Rischi:** API di `ironrdp-client` giovane (0.1.0, documentazione non generata); prestazioni del passaggio dei frame
via IPC (da misurare: 1080p a 30 fps con aggiornamenti parziali); manca H.264.

---

## 12. Build e distribuzione

- `tauri build` (da `frontend/` con `npm run tauri build -- --bundles app,dmg`) produce
  `target/release/bundle/macos/Portolano.app` e `…/dmg/Portolano_0.1.0_aarch64.dmg`. Il workspace condivide
  `target/` alla radice **[percorso esatto da verificare]**.
- **Firma.** La doc dice: "Ad-hoc code signing … is useful on ARM (Apple Silicon) devices, where code-signing is
  required for all apps from the Internet" (`"signingIdentity": "-"`), e "Ad-hoc code signing does not prevent MacOS
  from requiring users to whitelist the installation".
  - **Uso personale, build locale:** basta la firma ad hoc. Un'app compilata in locale non ha l'attributo di
    quarantena e Gatekeeper non la blocca.
  - **Distribuzione ad altri:** servono **Developer ID Application** (account Apple Developer a pagamento) e
    **notarizzazione** (`APPLE_ID` + `APPLE_PASSWORD` app-specific + `APPLE_TEAM_ID`, oppure API key
    `APPLE_API_ISSUER`/`APPLE_API_KEY`/`APPLE_API_KEY_PATH`). "Notarization is required when using a Developer ID
    Application certificate." Il bundler firma sidecar e app "inside out" e poi notarizza; `hardenedRuntime` è attivo
    per default. Dal codice letto non risultano entitlement necessari per WKWebView, perché JS gira nel processo
    WebContent di sistema **[da verificare con la prima build firmata]**.
- **Keychain:** l'ACL degli elementi del portachiavi (backend `keyring` "v1") lega l'accesso alla **firma del
  binario**.
  - Con la firma ad hoc il requisito designato è l'hash del codice: **cambia a ogni build**, e macOS richiede
    l'autorizzazione dopo ogni aggiornamento (già documentato nel README per la CLI).
  - Con Developer ID il requisito è identifier + Team ID, **stabile fra le versioni**.
  - App e sidecar sono **due binari distinti**: la CLI che legge il Keychain (nuova finestra, uso da terminale) chiede
    un proprio "Consenti sempre". Il client askpass non tocca il Keychain, perché il segreto lo serve il processo
    dell'app tramite socket.
  - **Conclusione:** per uso personale la firma ad hoc va bene, sapendo che il prompt torna dopo ogni build; per
    distribuire servono Developer ID e notarizzazione.
- **Dimensione attesa (stima da misurare):** binario dell'app in release ~10–20 MB (core, ops, import, keyring,
  argon2, zip, portable-pty, tauri e wry; WebKit è di sistema) + sidecar CLI **7,07 MB (misurato)** + asset frontend
  (~1–3 MB con font e xterm) ⇒ `.app` di circa 20–30 MB, `.dmg` compresso di circa 10–15 MB. Per ridurre:
  `[profile.release]` con `lto = true`, `codegen-units = 1`, `strip = true`, `panic` invariato, perché i test usano
  `should_panic` **[da valutare]**.

**Fonti:** https://v2.tauri.app/distribute/sign/macos/ · https://v2.tauri.app/reference/config/ (bundle.macOS) ·
https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/macos/app.rs · README del repo
("Prompt del portachiavi dopo ogni ricompilazione")

---

## Tabella riassuntiva delle dipendenze proposte

### Rust (`crates/portolano-app`)
| Crate | Versione | Scopo |
|---|---|---|
| `tauri` | 2.12.5 | runtime dell'app (feature `test` solo in dev-dependencies; `isolation` se adottato) |
| `tauri-build` | 2.7.4 | build script (AppManifest dei comandi) |
| `tauri-plugin-dialog` | 2.8.4 | dialoghi apri/salva/messaggio (usati da Rust) |
| `tauri-plugin-opener` | 2.7.3 | aprire link http(s) nel browser (xterm web-links, aiuto) |
| `tauri-plugin-window-state` | 2.5.0 | facoltativo: ricordare dimensione e posizione |
| `tauri-plugin-single-instance` | 2.5.5 | facoltativo: una sola istanza, apertura di file associati |
| `tauri-specta` | =2.0.0-rc.28 | bindings TS di comandi ed eventi (features `derive`, `typescript`) |
| `specta` | =2.0.0-rc.28 | derive `Type` (features `derive`, `uuid`, `chrono`) |
| `specta-typescript` | 0.0.15 | esportatore TypeScript |
| `portable-pty` | 0.9.0 | PTY per `ssh` |
| `notify` | 8.2.2 | osservazione del file del documento |
| `notify-debouncer-full` | 0.7.0 | debounce degli eventi |
| `tauri-plugin-wdio-webdriver` | 1.5.1 | solo con la feature `e2e`, mai in release |
| (esistenti) `portolano-ops`, `-core`, `-connectors`, `-import`, `serde`, `serde_json`, `thiserror`, `uuid`, `tracing` | workspace | |

Esclusi di proposito: `tauri-plugin-shell`, `tauri-plugin-fs`, `tauri-plugin-http`, `window-vibrancy` (sostituito da
`windowEffects`), `ts-rs` (ripiego), `ironrdp*` (fase successiva).

Piccole modifiche ai crate esistenti emerse dalla ricognizione:
1. `LazyVault::lock()`;
2. `Store::changed_on_disk()`;
3. una costante pubblica con le variabili d'ambiente da togliere ai figli (`CHILD_ENV_REMOVE`);
4. `OpsError::category()` condivisa fra CLI e app;
5. `Deserialize` sugli input di ops, oppure DTO nell'app.

### npm (`frontend/`)
| Pacchetto | Versione | Scopo |
|---|---|---|
| `@tauri-apps/api` | 2.12.4 | invoke, Channel, eventi, mocks |
| `@tauri-apps/cli` | 2.12.4 (dev) | `tauri dev/build/icon` |
| `@tauri-apps/plugin-dialog` | 2.8.4 | solo se i dialoghi si aprono da JS |
| `@tauri-apps/plugin-opener` | 2.7.3 | aprire link esterni |
| `react`, `react-dom` | 19.3.1 | UI |
| `@types/react`, `@types/react-dom` | 19.3.0 (dev) | tipi |
| `vite` | 8.3.8 (dev) | bundler / dev server |
| `@vitejs/plugin-react` | 6.3.0 (dev) | React in Vite |
| `typescript` | ~6.0.3 (dev) | **non la 7.x** (typescript-eslint) |
| `@tanstack/react-query` | 5.107.1 | dati del backend e cache |
| `zustand` | 5.1.0 | stato della UI |
| `@headless-tree/core`, `@headless-tree/react` | 1.8.0 | albero con DnD e a11y |
| `@tanstack/react-virtual` | 3.14.26 | virtualizzazione |
| `react-hook-form` | 7.92.0 | form |
| `zod` | 4.6.7 | schemi dei form |
| `@hookform/resolvers` | 7.0.5 | ponte RHF↔zod |
| `@xterm/xterm` | 6.0.0 | terminale |
| `@xterm/addon-fit` | 0.11.0 | adattamento alle dimensioni |
| `@xterm/addon-web-links` | 0.12.0 | link cliccabili |
| `@xterm/addon-webgl` | 0.19.0 | facoltativo, renderer GPU |
| `@fontsource/hanken-grotesk`, `@fontsource/ibm-plex-mono` | 5.3.0 | font locali al posto di Google Fonts |
| `eslint` | 10.14.0 (dev) | lint |
| `@eslint/js` | 10.0.1 (dev) | regole base |
| `typescript-eslint` | 8.72.2 (dev) | lint con i tipi |
| `eslint-plugin-react-hooks` | 7.1.1 (dev) | regole degli hook |
| `eslint-plugin-react-refresh` | 0.5.7 (dev) | HMR sicuro |
| `prettier` | 3.10.0 (dev) | formattazione |
| `vitest` | 5.1.0 (dev) | unit/component test |
| `jsdom` | 31.0.0 (dev) | DOM per i test |
| `@testing-library/react` | 16.3.3 (dev) | component test |
| `@testing-library/dom` | 10.4.2 (dev) | peer |
| `@testing-library/user-event` | 14.6.9 (dev) | interazioni |
| `@testing-library/jest-dom` | 7.0.1 (dev) | matcher |
| `@playwright/test` | 1.65.0 (dev) | E2E del frontend su WebKit con IPC simulato |
| `@wdio/tauri-service`, `@wdio/cli`, `webdriverio` | 1.5.1 / 10.2.0 / 10.2.0 (dev) | facoltativi: E2E sull'app vera |

## Cosa va installato sulla macchina
1. **Node 24 LTS** (24.22.3 "Krypton") con nvm: `nvm install 24` e `.nvmrc` con `24` in `frontend/`. In alternativa
   va bene la **v22.23.2 già installata** (`nvm use 22`), che soddisfa Vitest 5.1, jsdom 31 e Vite 8. **La v20.19.2
   predefinita non basta.**
2. **Le dipendenze npm** del progetto, in locale (`npm install` in `frontend/`). Nessun pacchetto globale; la CLI di
   Tauri arriva da `@tauri-apps/cli`, senza `cargo install tauri-cli`.
3. **Playwright WebKit** (`npx playwright install webkit`), solo se si adottano gli E2E Playwright.
4. **Niente da brew e niente Xcode completo:** le Command Line Tools 27.2 bastano per il desktop; Rust 1.99.1 c'è già
   (MSRV di Tauri 1.90, di IronRDP 1.94).
5. Solo per distribuire ad altri: un account Apple Developer (Developer ID Application) e le credenziali di
   notarizzazione. Non serve per l'uso personale.

## Punti incerti da chiudere con un prototipo (WP0 della FASE 2)
1. che `tauri-build` legga una config passata con `--config` (per tenere `externalBin` fuori dalle build del
   workspace); in alternativa, uno script che prepara il sidecar prima di ogni `cargo` del workspace;
2. che il fallback askpass su `/dev/tty` funzioni dentro la PTY di portable-pty (host key, prompt non riconosciuti);
3. se `style-src` possa fare a meno di `'unsafe-inline'` con xterm 6 (renderer DOM e WebGL);
4. la visualizzazione di TIFF e PDF in `<img>` da uno schema URI personalizzato in WKWebView;
5. i nomi esatti dei permessi dei plugin e della modalità d'errore di tauri-specta (`ErrorHandlingMode::Result`);
6. le prestazioni di `backdrop-filter` con sfondo animato e terminale aperto (Web Inspector);
7. la dimensione reale del bundle e il comportamento del prompt del Keychain per app e sidecar firmati ad hoc.
