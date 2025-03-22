# `cloud.ru` Advanced API in Rust

# Конфигурация

## Значения по умолчанию
```rust
DEFAULT_CREDENTIALS_FILE = "~/.cloudru/credentials";
DEFAULT_CONFIG_FILE = "~/.cloudru/config";
DEFAULT_CREDENTIAL = "default";
DEFAULT_LOG_LEVEL = "INFO";
ACCESS_KEY_ID_KEY = "access_key_id";
SECRET_ACCESS_KEY_KEY = "secret_access_key";
```
Приложение поддерживает настройку всех перечисленных параметров через переменные окружения, позволяя перезаписывать значения по умолчанию пользовательскими настройками.

# OBS

По умолчанию используется директория `~/.cloudru`. Для корректной работы в этой директории должен находиться файл `credentials` без расширения, но с содержимым в формате `.ini`.

Пример содержимого файла `credentials`:

```ini
[default]
access_key_id=YOUR_ACCESS_KEY
secret_access_key=YOUR_SECRET_KEY
```

## Параметры настройки

Настройки можно передать через следующие параметры:

| Переменная             | Описание                      |
| ---------------------- | ----------------------------- |
| `AK`                   | Access key.                   |
| `SK`                   | Secret key.                   |
| `SCA_CONFIG_FILE`      | Путь к файлу конфигурации.    |
| `SCA_CREDENTIALS_FILE` | Путь к файлу учетных данных.  |
| `SCA_CREDENTIALS_ID`   | Идентификатор учетных данных. |
| `SCA_REGION`           | Идентификатор региона SCA.    |
| `SCA_PROJECT_ID`       | Идентификатор проекта SCA.    |

Эти параметры могут быть заданы как через переменные окружения, так и через параметры запуска, что позволяет гибко настраивать приложения и перезаписывать значения по умолчанию. Значения необходимо скорректировать с учётом env_prefix и env_flavor_prefix, которые определяются в момент создания клиента  (например параметром SERVICE_ID).

## DLI
Минимальный конфиг файл `~/.cloudru/config` настроек для запуска
```ini
[common]
project_id="project_id"

[endpoint]
dli="https://dli_url"
```