# `cloud.ru` Advanced API in Rust

# Конфигурация

## значения по умолчанию
```rust
DEFAULT_CREDENTIALS_FILE = "~/.cloudru/credentials";
DEFAULT_CONFIG_FILE = "~/.cloudru/config";
DEFAULT_CREDENTIAL = "default";
DEFAULT_LOG_LEVEL = "INFO";
ACCESS_KEY_ID_KEY = "access_key_id";
SECRET_ACCESS_KEY_KEY = "secret_access_key";
```

## OBS
По умолчанию используется директория `~/.cloudru`. Для корректной работы в этой директории должен находиться файл `credentials` без расширения, но с содержимым в формате `.ini`. 
Пример содержимого файла `credentials`:

```ini
[default]
access_key_id=YOUR_ACCESS_KEY
secret_access_key=YOUR_SECRET_KEY
```



## DLI
Минимальный конфиг файл `~/.cloudru/config` настроек для запуска
```ini
[common]
project_id="project_id"

[endpoint]
dli="https://dli_url"
```