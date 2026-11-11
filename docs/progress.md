# Stato di avanzamento

Aggiornato dal PM alla fine di ogni passo. Serve anche a riprendere il lavoro se la sessione si interrompe.

## Stato attuale (2026-11-10)

| Passo | Stato | Note |
|---|---|---|
| Prerequisiti | fatto | Rust 1.99.1, FreeRDP 3.33.1 (`sdl-freerdp`); `samples/` con 2 documenti, ignorato da git |
| WP0 – scheletro e contratto | **integrato** | merge `6e49dfd`; due giri di revisione indipendente; 30 test; gate verdi su `main` |
| WP1 – albero, percorsi, risoluzione | **integrato** | merge `93c03c2`; due giri di revisione; 157 test nel core; gate verdi su `main` |
| WP2 – store, migrazioni, icone, config | **integrato** | merge `400adf0`; tre giri di revisione; lock per operazione (DEC-016.1), symlink, gc sicuro, `MAX_DEPTH` (DEC-021) |
| WP3 – vault | **integrato** | merge `b569fb3`; due giri di revisione; lock inter-processo (DEC-017), zeroize di Argon2; core: 297 test + 1 ignorato (Keychain) |
| WP4 – connettori | **integrato** | merge `3df8c3a`; tre giri di revisione; 158 test; opzioni FreeRDP reali (DEC-020) |
| WP5 – importer Royal | **integrato** | merge `737c58c`; tre giri di revisione; 159 test; test sui campioni reali verde (DEC-019, DEC-021) |
| WP6 – `portolano-ops` + CLI | in corso | piano dell'API di `portolano-ops` da approvare prima dell'implementazione (DEC-018) |
| WP7 – README, smoke, sicurezza | da fare | |
| Verifica finale | da fare | |

## Come riprendere dopo un'interruzione
1. `git log --oneline` su `main` e `git branch` per vedere i branch dei WP non ancora integrati.
2. I worktree dei sottoagenti sono in `.claude/worktrees/`: un `git status` dentro ciascuno mostra il lavoro non
   committato.
3. Le decisioni prese sono in `docs/DECISIONS.md`; il processo è in `docs/PM-PROMPT.md` (con la modifica DEC-007:
   il PM procede in autonomia).
4. Per il WP in revisione: rieseguire i quality gate (SPEC §11.2) sul suo branch e ripetere la revisione
   indipendente prima del merge.

## Seguiti da fare dopo l'integrazione dell'ondata 1
- Test di WP2 sul segreto sentinella: usare `MemoryVault` (WP3) invece della verifica sul solo `SecretRef`.
- Verifica dal vivo del prompt di `ssh-add` (DEC-015) con agent e chiave temporanei.

## Lavori rimandati
- RDP integrato nell'app con IronRDP (DEC-013): dopo la UI della FASE 2.

### Esigenze della UI (FASE 2) aggiungibili senza rompere il core
Dall'analisi del mock in `docs/ui/` (vedi DEC-016 per quelle già decise):
- traccia dell'ereditarietà della credenziale (`resolve_trace`: da quale cartella arriva);
- conteggio degli utilizzi di una credenziale inclusi gli ereditati (con `resolve_credential` su `walk`);
- anteprima dell'import (conteggi e numero di password richieste) e prompter asincrono con totale ("3 / 12",
  "Salta tutte");
- dal report d'import al nodo (via `source_id`);
- motivi di `Partial` per campi Royal specifici, se richiesti;
- nuovi controlli di validazione (credenziale senza password, inline duplicate confrontando solo utente e dominio);
- icone della libreria Royal mappate sul set builtin e `IconStore::list()`;
- riepilogo host/porta dei nodi non supportati estratto dall'importer;
- preferenze di aspetto della UI in un file separato da `config.toml` (non come sezione di primo livello, DEC-011);
- annullamento di un'eliminazione con cancellazione differita dei segreti;
- `self_exe` per askpass e nuova finestra = binario CLI incluso nell'app (sidecar), non l'eseguibile Tauri;
- stato bloccato/sbloccato del vault gestito dalla UI;
- operazione "Duplica" che rigenera id e `SecretRef` (DEC-016.4);
- estensione del file del documento (`.portolano` nel mock, `portolano.json` nella SPEC) da decidere per
  l'associazione dei file;
- "utente proposto" per la modalità "chiedi all'avvio";
- segreti dell'editor in staging: tenuti in memoria come `Secret` fino al Salva, poi `vault.set` → `save` con
  rollback; una sostituzione usa un nuovo `SecretRef` e il vecchio si cancella dopo il salvataggio; "Cancella"
  differito al Salva (evita segreti orfani nel Keychain dopo Annulla);
- salvataggio dell'editor dopo `ModifiedExternally`: modifiche tenute come patch e riapplicate dopo il reload;
- "Ricontrolla" con il vault bloccato: una sola nota invece di un `missing_secret` per ogni password.
