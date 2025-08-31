Технологический стек: 
    Rust - язык программирования
    OpenSSL - библиотека для SSL/TLS
    Многопоточность - обработка соединений в параллельных потоках
    TCP/IP - сетевые коммуникации

Зависимости
```toml
[dependencies]
openssl = "0.10"
openssl-sys = "0.9"
```
Запуск проекта
```bash
# Сборка проекта
cargo build --release

# Запуск сервера
cargo run --release
```


# Для тестирования используется самоподписанный сертификат:
```bash
openssl genrsa -out key.pem 2048
openssl req -new -x509 -key key.pem -out cert.pem -days 365
```
