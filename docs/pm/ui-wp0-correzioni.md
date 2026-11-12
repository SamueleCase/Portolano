# Correzioni richieste a UI-WP0 dalla revisione indipendente (2026-11-12)

Branch: `worktree-agent-ab96bd840230a5f0d`, worktree `.claude/worktrees/agent-ab96bd840230a5f0d`. Giudizio del
revisore: pronto per il merge dopo queste correzioni. Decisioni collegate: DEC-028.

0. **Contatori `usize`** (importante): sui campi `usize` dei tipi di SPEC-UI §3.5 con derive specta
   (`DeletePreview`, `Entry.depth`, `AddOutcome`, `ImportReport`/`TypeCounts` e ogni altro conteggio)
   `#[cfg_attr(feature = "specta", specta(type = u32))]`; nell'app un test che registra ed esporta in TypeScript tutti
   i tipi di §3.5 e fallisce se uno non è esportabile.
1. **Icone** (importante), `crates/portolano-app/src/icons.rs`: rifiutare ciò che non è un file regolare
   (`symlink_metadata`, niente symlink); limite di dimensione (quello del core per le icone se esiste); SHA-256 dei
   byte uguale al nome, altrimenti 404 con `nosniff`. Test: symlink verso un file esterno, hash diverso, file troppo
   grande. Proprietà estesa a `crates/portolano-core/src/icons*` solo per rendere pubblici controllo dell'hash e mappa
   estensione→MIME di `ImageType`, riusati dall'app al posto dei duplicati.
2. **Capability**: togliere `dialog:allow-message`.
3. **`crates/portolano-app/src/error.rs`** (`details_of`): candidati di `Ambiguous` estratti anche da
   `OpsError::Connect(ConnectError::Core(..))` e da ogni variante che incapsula l'errore del core; test.
4. **Test PTY** (`crates/portolano-app/tests/askpass_tty.rs`): attendere l'eco spento con
   `pair.master.get_termios()` e timeout invece dei 500 ms fissi; nella sonda `"$SSH_ASKPASS" "$1" </dev/null
   2>/dev/null` (prova l'uso di `/dev/tty`); asserire che il segreto sentinella non compare nell'output; messaggio
   chiaro se manca il binario della CLI (`cargo build -p portolano-cli`).
5. `ErrorCode::NoDocument`: `#[expect(dead_code, reason = "...")]` al posto di `allow`.
6. `frontend/package.json`: `engines.node` `>=22.23.2`.
7. `Store::changed_on_disk`: un solo helper per NotFound→`DocumentMissing` (condiviso con `save`); directory del
   documento assente → `DocumentMissing`, con test.

Rimandati: eco spento prima del prompt nel client askpass → UI-WP3 (DEC-028.2).
