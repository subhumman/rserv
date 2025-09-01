#!/bin/bash

echo "Генерация самоподписанного SSL сертификата..."

if [ ! -d "certs" ]; then
    mkdir certs
    echo "Создана папка certs"
fi

echo "Генерация приватного ключа..."
openssl genrsa -out certs/key.pem 2048

echo "Генерация сертификата..."
openssl req -new -x509 -key certs/key.pem -out certs/cert.pem -days 365 -subj "/C=RU/ST=Moscow/L=Moscow/O=RSserver/OU=Development/CN=localhost"

if [ -f "certs/key.pem" ] && [ -f "certs/cert.pem" ]; then
    echo "Сертификат успешно создан!"
    echo "Файлы сохранены в папке certs/"
    echo "   - key.pem (приватный ключ)"
    echo "   - cert.pml (сертификат)"
else
    echo "Ошибка при создании сертификата!"
fi

echo ""
echo "ВАЖНО: Это самоподписанный сертификат!"
echo "   Браузер будет показывать предупреждение о безопасности"
echo "   Для продакшена использовать сертификат от доверенного CA"
