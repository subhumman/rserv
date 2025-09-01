# PowerShell скрипт для генерации сертификата
# запускать от имени администратора

Write-Host " Генерация самоподписанного SSL сертификата..." -ForegroundColor Green

# + папка для сертификатов если её нет
if (!(Test-Path "certs")) {
    New-Item -ItemType Directory -Path "certs"
    Write-Host "Создана папка certs" -ForegroundColor Yellow
}

# гненерация приватного ключа
Write-Host " Генерация приватного ключа..." -ForegroundColor Yellow
openssl genrsa -out certs/key.pem 2048

# генерация сертификата
Write-Host "Генерация сертификата..." -ForegroundColor Yellow
openssl req -new -x509 -key certs/key.pem -out certs/cert.pem -days 365 -subj "/C=RU/ST=Moscow/L=Moscow/O=RSserver/OU=Development/CN=localhost"

# проверка 
if (Test-Path "certs/key.pem" -and Test-Path "certs/cert.pem") {
    Write-Host "Сертификат успешно создан!" -ForegroundColor Green
    Write-Host "Файлы сохранены в папке certs/" -ForegroundColor Cyan
    Write-Host "   - key.pem (приватный ключ)" -ForegroundColor Cyan
    Write-Host "   - cert.pem (сертификат)" -ForegroundColor Cyan
} else {
    Write-Host "Ошибка при создании сертификата!" -ForegroundColor Red
}

Write-Host "`nВАЖНО: Это самоподписанный сертификат!" -ForegroundColor Red
Write-Host "   Браузер будет показывать предупреждение о безопасности" -ForegroundColor Yellow
Write-Host "   Для продакшена использовать сертификат от доверенного CA" -ForegroundColor Yellow
