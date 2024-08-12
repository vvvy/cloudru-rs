# `cloud.ru` Advanced API in Rust

# Конфигурация

По умолчанию используется директория `~/.cloudru`. Для корректной работы в этой директории должен находиться файл `credentials` без расширения, но с содержимым в формате `.ini`. 
Пример содержимого файла `credentials`:

```ini
[default]
access_key_id=YOUR_ACCESS_KEY
secret_access_key=YOUR_SECRET_KEY
```