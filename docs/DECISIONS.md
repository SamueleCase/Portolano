# Decisioni

Registro delle decisioni prese durante l'implementazione: domande poste all'utente e deviazioni o
chiarimenti rispetto a `SPEC.md`. Le voci più recenti stanno in fondo.

---

## 2026-11-10 – Prerequisiti e contratto di WP0

Risposta dell'utente: "Va bene b". Interpretata come approvazione di tutte le raccomandazioni proposte
(opzione B per i punti 2–4, "sì" per i punti 1 e 5).

### DEC-001 – Installazione della toolchain
- **Domanda**: rustup/cargo e FreeRDP non sono installati; si possono installare con brew?
- **Decisione**: sì. Installati `rustup` (Rust stable 1.99.1) e `freerdp` 3.33.1 (binario `sdl-freerdp`).
- **Motivo**: Rust è necessario per l'implementazione; FreeRDP serve a verificare `/from-stdin` (SPEC §7.3).
- **Nota**: il formula `rustup` di brew è keg-only: i comandi si eseguono con
  `PATH=/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH`. Il `~/.zshrc` dell'utente non è stato modificato.

### DEC-002 – `resolve_parent_path` e opzione `-p` (SPEC §4.2)
- **Domanda**: la firma `resolve_parent_path(&self, path)` non può creare cartelle e non ha il parametro
  `create_missing` citato nel testo.
- **Decisione**: due metodi su `Document`:
  - `resolve_parent_path(&self, path: &str) -> Result<(Option<Uuid>, String)>`: sola lettura;
  - `ensure_parent_path(&mut self, path: &str) -> Result<(Option<Uuid>, String)>`: crea le cartelle
    intermedie mancanti (usato da `add … -p`).
- **Motivo**: la risoluzione in sola lettura resta disponibile su un documento immutabile (`show`, `connect`, UI).

### DEC-003 – `AppConfig` e indipendenza del core dai protocolli (SPEC §1.5 e §9)
- **Domanda**: le sezioni `[ssh]` e `[rdp]` tipizzate nel core violerebbero la regola per cui un nuovo
  protocollo non modifica il core.
- **Decisione**: il core tipizza solo `default_document`, `[vault]` e `[terminal]`. Le altre sezioni di primo
  livello (`[ssh]`, `[rdp]`, …) sono conservate come tabelle TOML grezze (`BTreeMap<String, toml::Value>`) e
  ogni `Protocol` deserializza la propria (`SshConfig`, `RdpConfig` in `portolano-connectors`).
  Il formato di `config.toml` resta quello del §9.
- **Motivo**: rispetta il §1.5 senza cambiare nulla per l'utente. `config set` sulle sezioni dei protocolli è
  validato dal protocollo, non dal core.

### DEC-004 – Comando per `connect --new-window` (SPEC §7.1, §7.2)
- **Domanda**: `PlanContext` non contiene il percorso del documento né le opzioni di vault/config che il
  processo figlio deve ricevere.
- **Decisione**: `ConnectOptions` riceve il campo `relaunch_args: Vec<String>`. La CLI lo riempie con le
  opzioni globali effettivamente specificate (`--file`, `--vault`, `--vault-file`, `--config`); il piano lo
  antepone a `connect @<id>`.
- **Motivo**: la nuova finestra usa esattamente lo stesso documento, vault e config del comando che l'ha
  aperta, e `plan` resta puro. Questi valori non sono segreti.

### DEC-005 – Dettagli di WP0 non fissati dalla SPEC
- **Decisione**:
  - Rust **edition 2024**, versione minima = stable corrente (`rust-version` nel workspace);
  - i tipi del §3 senza `derive` esplicito nella SPEC hanno gli stessi derive di `Document`
    (`Debug, Clone, PartialEq, Serialize, Deserialize`), più `Default` dove la SPEC lo indica;
  - crate con `publish = false` e senza campo `license`.

### DEC-006 – Conflitti fra documenti già risolti dalla SPEC (nessuna domanda)
- `CredentialMode` assente → `Inherit` (SPEC §8.3), non `none` come in `royal-format.md` §5.1.
- Service Keychain `dev.portolano` (SPEC §5.2), non `portolano`.
