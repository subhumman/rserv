[CmdletBinding()]
param (
    [string]$CertDir = "certs",
    [int]$ValidityDays = 365,
    [string]$Subject = "/C=RU/ST=Moscow/L=Moscow/O=RServer/CN=localhost"
)

$ErrorActionPreference = "Stop"

# 1. Проверка OpenSSL
if (!(Get-Command openssl -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: OpenSSL not found. Please install it." -ForegroundColor Red
    return
}

# 2. Создание папки
if (!(Test-Path $CertDir)) {
    New-Item -ItemType Directory -Path $CertDir | Out-Null
    Write-Host "[+] Folder $CertDir created" -ForegroundColor Green
}

$KeyPath = Join-Path $CertDir "key.pem"
$CertPath = Join-Path $CertDir "cert.pem"

try {
    Write-Host "[*] Generating RSA key..." -ForegroundColor Yellow
    & openssl genrsa -out $KeyPath 2048

    Write-Host "[*] Generating X.509 cert..." -ForegroundColor Yellow
    & openssl req -new -x509 -key $KeyPath -out $CertPath -days $ValidityDays -subj $Subject

    if ((Test-Path $KeyPath) -and (Test-Path $CertPath)) {
        Write-Host "DONE: Certificates generated successfully!" -ForegroundColor Green
    }
}
catch {
    Write-Host "CRITICAL ERROR: $($_.Exception.Message)" -ForegroundColor Red
}