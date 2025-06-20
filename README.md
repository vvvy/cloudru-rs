# `cloud.ru` Advanced API in Rust

# Конфигурация (быстрый старт)

Конфигурация библиотеки доступна как через конфигурационные файлы (рекомендованный путь), так и программным способом.
Здесь описана конфигурация через конфигурационные файлы.

По умолчанию для конфигурационных файлов используется директория `~/.cloudru`.  

В этой директории должен находиться файл `credentials` с содержимым в формате `.ini`. 
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

Минимальный конфиг файл `~/.cloudru/config` настроек для DLI:

```ini
[common]
project_id=YOUR_PROJECT_ID
```