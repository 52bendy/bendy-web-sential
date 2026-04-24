#!/bin/bash
#
# bendy-web-sential Backup Script
# Performs: SQLite DB backup + TOTP AES key backup (if configured)
# Usage: ./scripts/backup.sh [--restore <backup-file>] [--list]
#

set -e

PROJECT_ROOT="/myproject/rust/bendy-web-sential"
BACKUP_DIR="${BWS_BACKUP_DIR:-$PROJECT_ROOT/backups}"
TIMESTAMP=$(date '+%Y%m%d_%H%M%S')
HOSTNAME=$(hostname)

mkdir -p "$BACKUP_DIR"

do_backup() {
    echo "=== bendy-web-sential Backup ==="
    echo "Timestamp: $TIMESTAMP"
    echo "Backup dir: $BACKUP_DIR"
    echo ""

    # Source .env to get database URL
    if [ -f "$PROJECT_ROOT/.env" ]; then
        set -a
        source "$PROJECT_ROOT/.env"
        set +a
    fi

    local db_path="${BWS_DATABASE_URL#sqlite://}"
    local db_name=$(basename "$db_path")
    local db_backup="$BACKUP_DIR/bws_db_${TIMESTAMP}.sqlite"

    # Backup SQLite database
    echo "[1/2] Backing up database ($db_path)..."
    if [ -f "$db_path" ]; then
        cp "$db_path" "$db_backup"
        echo "  -> $db_backup ($(du -h "$db_backup" | cut -f1))"
    else
        echo "  WARNING: Database file not found at $db_path"
    fi

    # Backup TOTP AES key
    echo ""
    echo "[2/2] Backing up TOTP AES key..."
    if [ -n "$BWS_TOTP_AES_KEY" ]; then
        local key_backup="$BACKUP_DIR/bws_totp_key_${TIMESTAMP}.enc"
        echo "$BWS_TOTP_AES_KEY" | openssl enc -aes-256-cbc -salt -pbkdf2 -out "$key_backup" -pass pass:"${BWS_BACKUP_ENCRYPTION_PASSPHRASE:-change-me}"
        echo "  -> $key_backup (encrypted)"
    else
        echo "  SKIP: BWS_TOTP_AES_KEY not set"
    fi

    # Cleanup old backups beyond retention
    local retention_days="${BWS_BACKUP_RETENTION_DAYS:-7}"
    echo ""
    echo "[+] Cleaning up backups older than $retention_days days..."
    find "$BACKUP_DIR" -name "bws_db_*.sqlite" -mtime "+$retention_days" -delete
    find "$BACKUP_DIR" -name "bws_totp_key_*.enc" -mtime "+$retention_days" -delete
    echo "  Done."

    echo ""
    echo "=== Backup Complete ==="
}

do_list() {
    echo "=== Available Backups ==="
    echo ""

    echo "Database backups:"
    if ls "$BACKUP_DIR"/bws_db_*.sqlite &>/dev/null; then
        for f in "$BACKUP_DIR"/bws_db_*.sqlite; do
            local size=$(du -h "$f" | cut -f1)
            local age=$(stat -c '%y' "$f" 2>/dev/null | cut -d' ' -f1,2 | cut -d'.' -f1)
            echo "  $(basename "$f")  $size  $age"
        done
    else
        echo "  (none)"
    fi

    echo ""
    echo "TOTP key backups (encrypted):"
    if ls "$BACKUP_DIR"/bws_totp_key_*.enc &>/dev/null; then
        for f in "$BACKUP_DIR"/bws_totp_key_*.enc; do
            local size=$(du -h "$f" | cut -f1)
            local age=$(stat -c '%y' "$f" 2>/dev/null | cut -d' ' -f1,2 | cut -d'.' -f1)
            echo "  $(basename "$f")  $size  $age"
        done
    else
        echo "  (none)"
    fi
    echo ""
}

do_restore() {
    local backup_file="$1"
    local passphrase="${BWS_BACKUP_ENCRYPTION_PASSPHRASE:-change-me}"

    if [ -z "$backup_file" ]; then
        echo "Usage: $0 --restore <backup-file>"
        exit 1
    fi

    if [ ! -f "$backup_file" ]; then
        echo "ERROR: Backup file not found: $backup_file"
        exit 1
    fi

    echo "=== bendy-web-sential Restore ==="
    echo "Backup file: $backup_file"
    echo ""

    if [[ "$backup_file" == *.sqlite ]]; then
        # Source .env
        if [ -f "$PROJECT_ROOT/.env" ]; then
            set -a
            source "$PROJECT_ROOT/.env"
            set +a
        fi

        local db_path="${BWS_DATABASE_URL#sqlite://}"
        local db_dir=$(dirname "$db_path")

        echo "[1/2] Restoring database..."
        echo "  From: $backup_file"
        echo "  To:   $db_path"

        # Create backup of current DB before restore
        if [ -f "$db_path" ]; then
            local pre_backup="$db_path.pre_restore_$(date '+%Y%m%d_%H%M%S')"
            cp "$db_path" "$pre_backup"
            echo "  Pre-restore backup: $pre_backup"
        fi

        cp "$backup_file" "$db_path"
        echo "  Done."

        echo ""
        echo "[2/2] Restoring TOTP key..."
        local totp_backup=$(echo "$backup_file" | sed 's|bws_db_|bws_totp_key_|;s|\.sqlite$|_totp\.enc|')
        # Try to find matching TOTP backup by timestamp
        local ts=$(echo "$backup_file" | sed 's|.*bws_db_||;s|\.sqlite$||')
        totp_backup="$BACKUP_DIR/bws_totp_key_${ts}.enc"

        if [ -f "$totp_backup" ]; then
            local decrypted=$(openssl enc -aes-256-cbc -d -pbkdf2 -in "$totp_backup" -pass pass:"$passphrase" 2>/dev/null)
            if [ -n "$decrypted" ]; then
                echo "  -> TOTP key found and verified"
                echo "  NOTE: Update .env with: BWS_TOTP_AES_KEY=$decrypted"
            else
                echo "  WARNING: Could not decrypt TOTP backup (wrong passphrase?)"
            fi
        else
            echo "  SKIP: No matching TOTP backup found"
        fi

    elif [[ "$backup_file" == *.enc ]]; then
        echo "Extracting TOTP key from encrypted backup..."
        local decrypted=$(openssl enc -aes-256-cbc -d -pbkdf2 -in "$backup_file" -pass pass:"$passphrase" 2>/dev/null)
        if [ -n "$decrypted" ]; then
            echo "BWS_TOTP_AES_KEY=$decrypted"
            echo ""
            echo "To update your .env, run:"
            echo "  sed -i 's|BWS_TOTP_AES_KEY=.*|BWS_TOTP_AES_KEY=$decrypted|' $PROJECT_ROOT/.env"
        else
            echo "ERROR: Could not decrypt (wrong passphrase?)"
            exit 1
        fi
    else
        echo "ERROR: Unknown backup file type: $backup_file"
        exit 1
    fi

    echo ""
    echo "=== Restore Complete ==="
    echo "Restart the service to apply changes."
}

case "${1:-}" in
    --restore|-r)
        do_restore "$2"
        ;;
    --list|-l)
        do_list
        ;;
    ""|backup)
        do_backup
        ;;
    help|--help|-h)
        cat << EOF
bendy-web-sential Backup Script

Usage: $0 <command>

Commands:
    (default)         Perform backup
    --restore, -r     Restore from backup
    --list, -l        List available backups
    help              Show this help

Environment variables:
    BWS_BACKUP_DIR              Backup directory (default: ./backups)
    BWS_BACKUP_RETENTION_DAYS   Days to keep backups (default: 7)
    BWS_BACKUP_ENCRYPTION_PASSPHRASE  Passphrase for TOTP key encryption

Examples:
    $0                      # Create backup
    $0 --list               # List backups
    $0 --restore backups/bws_db_20240115_120000.sqlite  # Restore DB
EOF
        ;;
    *)
        echo "Unknown command: $1"
        echo "Run '$0 help' for usage"
        exit 1
        ;;
esac
