# --- Этап 1: Сборка (Builder) ---
FROM rust:1.75-slim AS builder

# Ставим зависимости для компиляции
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Копируем всё содержимое проекта
COPY . .

# Проверяем, что файлы реально попали в builder (увидишь в логах при сборке)
RUN ls -la index.html 404.html certs/

# Собираем бинарник
RUN cargo build --release

# --- Этап 2: Финальный образ (Runtime) ---
FROM debian:bookworm-slim

# Ставим библиотеки для запуска
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем бинарник. 
# ВНИМАНИЕ: убедись, что имя файла в target/release/ совпадает с названием проекта в Cargo.toml
# Если проект называется rserv, то файл будет rserv. Если rserver - то rserver.
COPY --from=builder /usr/src/app/target/release/rserver ./rserver

# Копируем статику и сертификаты по одному для надежности
COPY --from=builder /usr/src/app/index.html ./index.html
COPY --from=builder /usr/src/app/404.html ./404.html
COPY --from=builder /usr/src/app/certs ./certs

# Настройка прав (хороший тон для безопасности)
RUN chmod +x ./rserver

EXPOSE 8443

# Используем флаг --https, как в твоем Main.rs
CMD ["./rserver", "--https"]