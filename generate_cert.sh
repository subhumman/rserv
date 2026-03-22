#!/usr/bin/env bash

# стрикт-режим:
# -e выход при ошибке любой команды
# -u выход при использовании необъявленной переменной
# -o pipefail:  ошибка в пайпе (A | B) считается ошибкой всей строки
set -euo pipefail

# константы
CERT_DIR="certs"
DAYS=365
SUBJ="/C=RU/ST=Moscow/L=Moscow/O=RSserver/OU=Development/CN=localhost"

# цвета для вывода
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly NC='\033[0m' # без цвета

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

main() {
    log_info "Начало генерации SSL сертификата..."

    # 1 проверка зависимостей
    if ! command -v openssl &> /dev/null; then
        log_error "OpenSSL не найден. Установите его перед запуском."
        exit 1
    fi

    # 2 создание директории
    if [[ ! -d "$CERT_DIR" ]]; then
        mkdir -p "$CERT_DIR"
        log_info "Создана директория: $CERT_DIR"
    fi

    # 3 генерация ключа и сертификата
    local key_path="${CERT_DIR}/key.pem"
    local cert_path="${CERT_DIR}/cert.pem"

    log_info "Генерация приватного ключа..."
    if openssl genrsa -out "$key_path" 2048 2>/dev/null; then
        log_info "Генерация сертификата..."
        openssl req -new -x509 -key "$key_path" -out "$cert_path" -days "$DAYS" -subj "$SUBJ" 2>/dev/null
    else
        log_error "Ошибка при вызове openssl."
        exit 1
    fi

    # 4 итоговая проверка
    if [[ -f "$key_path" && -f "$cert_path" ]]; then
        log_info "Сертификаты успешно созданы в ${CERT_DIR}/"
        echo -e "   - ${key_path}"
        echo -e "   - ${cert_path}"
    else
        log_error "Файлы не были созданы!"
        exit 1
    fi

    echo -e "\n${RED}ВАЖНО:${NC} Это самоподписанный сертификат для разработки."
}

# запуск основной функции
main "$@"