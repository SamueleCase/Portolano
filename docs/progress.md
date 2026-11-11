# Stato di avanzamento

Aggiornato dal PM alla fine di ogni passo. Serve anche a riprendere il lavoro se la sessione si interrompe.

## Stato attuale (2026-11-10)

| Passo | Stato | Note |
|---|---|---|
| Prerequisiti | fatto | Rust 1.99.1, FreeRDP 3.33.1 (`sdl-freerdp`); `samples/` con 2 documenti, ignorato da git |
| WP0 – scheletro e contratto | **integrato** | merge `6e49dfd`; due giri di revisione indipendente; 30 test; gate verdi su `main` |
| WP1…WP5 – ondata 1 | in corso | 5 sottoagenti in parallelo, ciascuno in un worktree sotto `.claude/worktrees/`; merge previsto in ordine WP1 → WP5 |
| WP6 – CLI | da fare | |
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

## Lavori rimandati
- RDP integrato nell'app con IronRDP (DEC-013): dopo la UI della FASE 2.
