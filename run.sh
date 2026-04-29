#!/bin/bash
set -e

# Nome del binario
BIN_NAME="f-run"

# Percorso destinazione
# Prova a leggere la cartella download ufficiale, se fallisce usa quella di default
DEST=$(xdg-user-dir DOWNLOAD 2>/dev/null || echo "$HOME/Downloads")

usage() {
    echo "Utilizzo: $0 [-h] [-s <numero_script>] [-f]"
    echo "  -h  Mostra aiuto"
    echo "  -s  Specifica il numero dello script da eseguire"
    exit 1
}

script_number=""
VERSION=""

while getopts ":hs:f" opt; do
    case ${opt} in
        h)
            usage
            ;;
        s)
            script_number=$OPTARG
            ;;
        \?)
            echo "Invalid option: -$OPTARG" 1>&2
            usage
            ;;
        :)
            echo "Invalid option: -$OPTARG requires an argument" 1>&2
            usage
            ;;
    esac
done
shift $((OPTIND -1))

if [ -z "$script_number" ]; then
    echo "Quale script vuoi eseguire: "
    echo "1) Build"
    echo "2) Apri documentazione"
    echo "3) Analizza"
    read script_number
fi

function build_artifacts() {
    echo "🔧 Building release binary..."
    
    # Estraiamo la versione dal Cargo.toml
    VERSION=$(grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f2)
    echo "📌 Version: $VERSION"

    # Compiliamo per la piattaforma corrente
    cargo build --release

    # Rileviamo automaticamente dove Cargo ha messo il file
    # 'target/release/' è il default quando non si specifica un --target particolare
    SOURCE_PATH="./target/release/$BIN_NAME"

    # Gestione Windows: aggiunge .exe se il file esiste con quell'estensione
    if [ -f "${SOURCE_PATH}.exe" ]; then
        SOURCE_PATH="${SOURCE_PATH}.exe"
        BIN_DEST_NAME="${BIN_NAME}.exe"
    else
        BIN_DEST_NAME="$BIN_NAME"
    fi

    if [ -f "$SOURCE_PATH" ]; then
        echo "📦 Moving binary to $DEST..."
        mv "$SOURCE_PATH" "$DEST/$BIN_DEST_NAME"
        echo "✅ Fatto!"
    else
        echo "❌ Errore: Binario non trovato in $SOURCE_PATH"
        exit 1
    fi
}

case $script_number in
    "1")
        build_artifacts
        ;;
    
    "2")
        cargo doc --open
        ;;
    
    "3") 
        cargo c && cargo clippy
        ;;

    *)
        echo "👻 Hai inserito un valore non valido 👻"
        ;;
esac