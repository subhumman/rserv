# PowerShell скрипт для генерации самоподписанного SSL сертификата
# Запускать от имени администратора

Write-Host "🔐 Генерация самоподписанного SSL сертификата..." -ForegroundColor Green

# Создаем папку для сертификатов если её нет
if (!(Test-Path "certs")) {
    New-Item -ItemType Directory -Path "certs"
    Write-Host "📁 Создана папка certs" -ForegroundColor Yellow
}

# Генерируем приватный ключ
Write-Host "🔑 Генерация приватного ключа..." -ForegroundColor Yellow
openssl genrsa -out certs/key.pem 2048

# Генерируем самоподписанный сертификат
Write-Host "📜 Генерация сертификата..." -ForegroundColor Yellow
openssl req -new -x509 -key certs/key.pem -out certs/cert.pem -days 365 -subj "/C=RU/ST=Moscow/L=Moscow/O=RSserver/OU=Development/CN=localhost"

# Проверяем созданные файлы
if (Test-Path "certs/key.pem" -and Test-Path "certs/cert.pem") {
    Write-Host "✅ Сертификат успешно создан!" -ForegroundColor Green
    Write-Host "📁 Файлы сохранены в папке certs/" -ForegroundColor Cyan
    Write-Host "   - key.pem (приватный ключ)" -ForegroundColor Cyan
    Write-Host "   - cert.pem (сертификат)" -ForegroundColor Cyan
} else {
    Write-Host "❌ Ошибка при создании сертификата!" -ForegroundColor Red
}

Write-Host "`n⚠️  ВАЖНО: Это самоподписанный сертификат!" -ForegroundColor Red
Write-Host "   Браузер будет показывать предупреждение о безопасности" -ForegroundColor Yellow
Write-Host "   Для продакшена используйте сертификат от доверенного CA" -ForegroundColor Yellow
