#!/usr/bin/env bash
# End-to-end smoke test of the `portolano` CLI (SPEC §11.4).
#
# Runs every step on a throwaway document, configuration and file vault in a
# temporary directory: it never touches the user's document, configuration,
# vault or Keychain, and it never launches a real client. `ssh`, FreeRDP,
# `ssh-add`, `osascript` and `open` are replaced by stubs that record any call;
# the test fails if one of them ran.
#
# Usage: scripts/smoke.sh
#   PORTOLANO_BIN=/path/to/portolano scripts/smoke.sh   to test a given binary
#
# Ends with "SMOKE OK"; on the first failure it prints the step, the command
# output and exits with 1.

set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
fixtures="$repo_root/crates/portolano-import/tests/fixtures"

# --- Binary -------------------------------------------------------------------

if [[ -n "${PORTOLANO_BIN:-}" ]]; then
    bin=$PORTOLANO_BIN
else
    if ! command -v cargo >/dev/null 2>&1; then
        # Homebrew's rustup is keg-only (see README).
        PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
    fi
    (cd "$repo_root" && cargo build -q -p portolano-cli)
    bin="$repo_root/target/debug/portolano"
fi
[[ -x "$bin" ]] || { echo "smoke: binario non trovato: $bin" >&2; exit 1; }

# --- Isolated environment -----------------------------------------------------

work=$(mktemp -d "${TMPDIR:-/tmp}/portolano-smoke.XXXXXX")
trap 'rm -rf "$work"' EXIT

doc="$work/portolano.json"
vault="$work/vault.json"
config="$work/config.toml"
calls="$work/client-calls.log"
outputs="$work/outputs.log"
stubs="$work/bin"

# Stubs for every external program a plan could start: they only record the
# call and fail. They come first in the PATH and are the configured clients.
mkdir -p "$stubs"
for program in ssh ssh-add ssh-agent osascript open security \
    sdl-freerdp3 sdl-freerdp xfreerdp3 xfreerdp; do
    cat >"$stubs/$program" <<EOF
#!/bin/sh
echo "$program \$*" >>"$calls"
exit 97
EOF
    chmod +x "$stubs/$program"
done
cat >"$config" <<EOF
[vault]
backend = "file"
file = "$vault"

[ssh]
program = "$stubs/ssh"

[rdp]
client = "freerdp"
freerdp_program = "$stubs/sdl-freerdp"
EOF

# A stale askpass session in the environment would turn the binary into an
# askpass client; the user's document and vault variables must not leak in.
unset PORTOLANO_ASKPASS PORTOLANO_ASKPASS_SOCK PORTOLANO_ASKPASS_TOKEN \
    PORTOLANO_FILE PORTOLANO_VAULT PORTOLANO_VAULT_FILE
export PORTOLANO_VAULT_PASSWORD="test-only-master-password"
# Default locations (data and cache directories) fall inside the temporary
# directory too, should anything ignore the explicit paths.
export HOME="$work/home"
mkdir -p "$HOME"
export PATH="$stubs:$PATH"
# No command may wait for an answer on the terminal.
exec </dev/null

# The secret of the test: it must never appear in any output or file.
sentinel="smoke-sentinel-$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')"

# --- Helpers ------------------------------------------------------------------

step_name=""
out=""

step() {
    step_name=$1
    printf '[%s] %s\n' "$2" "$step_name"
}

fail() {
    printf 'FALLITO (%s): %s\n' "$step_name" "$1" >&2
    if [[ -n "$out" ]]; then
        printf -- '--- output ---\n%s\n--------------\n' "$out" >&2
    fi
    exit 1
}

# portolano EXPECTED_EXIT ARGS...: runs the binary on the test document,
# vault and configuration; stdout and stderr end up in $out and are appended
# to $outputs, which is searched for the sentinel at every check. Only the
# command name goes to the log, not its arguments: one test passes the
# sentinel on purpose to a rejected option. Stdin is the caller's (a
# here-string where a secret is read).
portolano() {
    local expected=$1 code=0
    shift
    out=$("$bin" --file "$doc" --vault file --vault-file "$vault" --config "$config" "$@" 2>&1) ||
        code=$?
    printf '### portolano %s (exit %s)\n%s\n' "$1" "$code" "$out" >>"$outputs"
    [[ $code -eq $expected ]] ||
        fail "portolano $* -> exit $code, atteso $expected"
}

contains() {
    [[ "$out" == *"$1"* ]] || fail "manca nell'output: $1"
}

lacks() {
    [[ "$out" != *"$1"* ]] || fail "non deve comparire nell'output: $1"
}

file_lacks() {
    local text=$1
    shift
    local file
    for file in "$@"; do
        if [[ -f "$file" ]] && grep -qF -- "$text" "$file"; then
            fail "$file contiene un valore che non deve contenere"
        fi
    done
}

no_secret_anywhere() {
    file_lacks "$sentinel" "$outputs" "$doc" "$doc.bak" "$vault" "$config" "$work"/report-*.json
}

# --- Steps (SPEC §11.4) -------------------------------------------------------

step "init" 1
portolano 0 init --name Smoke
contains "Documento 'Smoke' creato"
[[ -f "$doc" ]] || fail "documento non creato"
portolano 1 init
contains "esiste già"

step "add folder -p" 2
portolano 0 add folder -p Clienti/ACME
contains "Aggiunto /Clienti/ACME"

step "add cred con password da stdin" 3
portolano 0 add cred "Clienti/ACME/Admin ACME" --user administrator --domain ACME \
    --password-stdin <<<"$sentinel"
contains "Aggiunto /Clienti/ACME/Admin ACME"
portolano 2 add cred Clienti/ACME/Altra --password "$sentinel"
no_secret_anywhere

step "add ssh con --cred" 4
portolano 0 add ssh Clienti/ACME/bastion --host bastion.example.com \
    --cred "Clienti/ACME/Admin ACME"
contains "Aggiunto /Clienti/ACME/bastion"

step "add ssh con --jump" 5
portolano 0 add ssh Clienti/ACME/web01 --host web01.example.com --port 2222 \
    --cred "Clienti/ACME/Admin ACME" --jump Clienti/ACME/bastion \
    --jump ops@edge.example.com:2200 --agent-forwarding
contains "Aggiunto /Clienti/ACME/web01"
portolano 0 --json show Clienti/ACME/web01
contains '"$ref"'
contains '"address": "ops@edge.example.com:2200"'

step "add rdp con gateway" 6
portolano 0 add rdp Clienti/ACME/dc01 --host 192.0.2.10 --cred "Clienti/ACME/Admin ACME" \
    --gateway rdg.example.com:443 --gateway-cred "Clienti/ACME/Admin ACME" \
    --resolution 1920x1080
contains "Aggiunto /Clienti/ACME/dc01"

step "tree, list, show --resolved" 7
portolano 0 tree
contains "Smoke"
contains "[ssh] web01"
contains "[rdp] dc01"
contains 'ACME\administrator@192.0.2.10'
portolano 0 list --type ssh
contains "/Clienti/ACME/bastion"
contains "administrator@web01.example.com:2222"
lacks "/Clienti/ACME/dc01"
portolano 0 show --resolved Clienti/ACME/web01
contains "Origine:  credenziale /Clienti/ACME/Admin ACME"
contains "Password: presente nel vault"
no_secret_anywhere

step "move e riordino" 8
portolano 0 add folder Clienti/Windows
portolano 0 move Clienti/ACME/dc01 Clienti/Windows
contains "Spostato in /Clienti/Windows/dc01"
portolano 0 move Clienti/ACME/web01 Clienti/ACME --position 0
portolano 0 list --type ssh Clienti/ACME
first_ssh=$(printf '%s\n' "$out" | sed -n 2p)
[[ "$first_ssh" == *"/Clienti/ACME/web01"* ]] || fail "web01 non è il primo dopo il riordino"
portolano 0 move Clienti/ACME/bastion Clienti/ACME --before Clienti/ACME/web01
portolano 0 list --type ssh Clienti/ACME
first_ssh=$(printf '%s\n' "$out" | sed -n 2p)
[[ "$first_ssh" == *"/Clienti/ACME/bastion"* ]] || fail "bastion non è il primo dopo --before"
portolano 1 move Clienti Clienti/ACME
contains "errore"

step "edit --set" 9
portolano 0 edit Clienti/ACME/web01 --set keepalive_s=30 --set compression=true --tag prod
portolano 0 --json show Clienti/ACME/web01
contains '"keepalive_s": 30'
contains '"compression": true'
portolano 4 edit Clienti/ACME/web01 --set campo_inesistente=1
contains "campo sconosciuto"

step "validate" 10
portolano 0 validate
contains "0 errori"

step "connect --print (ssh e rdp)" 11
portolano 0 connect --print Clienti/ACME/web01
contains "$stubs/ssh -p 2222 -l administrator"
contains "-J administrator@bastion.example.com:22,ops@edge.example.com:2200"
contains "-- web01.example.com"
contains "<vault:"
no_secret_anywhere
portolano 0 connect --print Clienti/Windows/dc01
contains "$stubs/sdl-freerdp /v:192.0.2.10:3389 /u:administrator /d:ACME"
contains "/gateway:g:rdg.example.com:443"
contains "/from-stdin:force"
contains "<vault:"
lacks "/p:"
no_secret_anywhere
[[ ! -e "$calls" ]] || fail "un client è stato avviato: $(cat "$calls")"

step "import delle fixture con --report" 12
portolano 0 add folder Import
for fixture in minimal workspace full-synthetic; do
    source_file="$fixtures/$fixture.royal.xml"
    report="$work/report-$fixture.json"
    portolano 0 import "$source_file" --into Import --report "$report"
    contains "RoyalDocument"
    [[ -s "$report" ]] || fail "report JSON non creato per $fixture"
    grep -q '"importer": "royal"' "$report" || fail "report JSON senza importer: $fixture"
    # Apart from the embedded icon, every base64 run of 40+ characters in a
    # fixture is Royal ciphertext: it must reach neither the report nor the
    # document (SPEC D3, §8.2).
    while IFS= read -r ciphertext; do
        file_lacks "$ciphertext" "$report" "$doc"
    done < <(grep -v '<CustomImage>' "$source_file" | grep -oE '[A-Za-z0-9+/]{40,}={0,2}' || true)
done
grep -q '"status": "partial"' "$work/report-workspace.json" ||
    fail "le password cifrate della fixture workspace non risultano parziali"
portolano 0 tree Import --depth 1
contains "Minimo"
contains "Workspace di prova"
contains "Sintetico completo"
portolano 1 import "$fixtures/minimal.royal.xml" --into Import
contains "esistono già"

step "delete -r" 13
portolano 1 delete Clienti/ACME -r -y
contains "eliminazione rifiutata"
portolano 0 delete Clienti/Windows -r -y
contains "Eliminato /Clienti/Windows"
portolano 0 delete Clienti/ACME -r -y
contains "Eliminato /Clienti/ACME: 4 nodi, 1 segreti cancellati dal vault"
portolano 3 show Clienti/ACME
# The synthetic fixtures carry deliberate errors (a credential outside the
# document): validate fails on them, but nothing is left dangling in Clienti.
portolano 4 validate
lacks "/Clienti"
no_secret_anywhere

step "secret status" 14
credential="Import/Workspace di prova/Credenziali/Deploy"
portolano 0 secret status "$credential"
contains "nessun segreto"
portolano 0 secret set "$credential" --stdin <<<"$sentinel"
portolano 0 secret status "$credential"
contains "password: presente nel vault"
PORTOLANO_VAULT_PASSWORD="not-the-master-password" portolano 5 secret status "$credential"
contains "master password errata"
no_secret_anywhere

# Beyond SPEC §11.4: the commands the steps above leave out.
step "secret clear" +
portolano 0 secret clear "$credential"
contains "Eliminato: password"
portolano 0 secret status "$credential"
contains "nessun segreto"

step "config path, show, set" +
portolano 0 config path
[[ "$out" == "$config" ]] || fail "config path non indica il file di configurazione di test"
portolano 0 config set rdp.cert deny
portolano 0 config show
contains 'cert = "deny"'
contains "freerdp_program = \"$stubs/sdl-freerdp\""
portolano 4 config set rdp.client nessuno
contains "valore non valido"
portolano 4 config set chiave_inesistente 1
no_secret_anywhere
[[ ! -e "$calls" ]] || fail "un programma esterno è stato avviato: $(cat "$calls")"

echo "SMOKE OK"
