# error-remapper

[![en](https://img.shields.io/badge/lang-English-blue)](README.en.md)

Консольная утилита на Rust для нечёткого поиска и подмены кодов ошибок бэкенд-систем по YAML-словарю.

## Зачем

Смежные бэкенд-системы возвращают ошибки в произвольном формате с разными кодами и текстами. Утилита принимает JSON с ошибкой, находит наиболее подходящую запись в YAML-словаре (по коду и/или нечётко по тексту) и возвращает унифицированный код и описание ошибки.

## Установка

```bash
git clone https://github.com/<your-username>/error-remapper.git
cd error-remapper
cargo build --release
```

Бинарный файл: `target/release/error-remapper`

## Быстрый старт

```bash
# Передать JSON аргументом
error-remapper '{"error":{"code":"3011","title":"Не пройден фрод-мониторинг"}}'

# Результат:
# {"code":"81005","customDesc":"Перевод отклонён банком получателя","matched":true}

# Передать JSON через stdin
echo '{"error":{"code":"2001","title":"Got unexpected symbol: @"}}' | error-remapper
```

## Алгоритм поиска

1. **Точное совпадение по коду** — если код ошибки из JSON совпадает с `key` в YAML и найдена ровно одна запись, она используется.
2. **Нечёткий поиск по тексту** — если точного совпадения нет (0 или несколько записей с одинаковым `key`):
   - Если записей с совпавшим кодом несколько — нечёткий поиск только среди них.
   - Если записей с совпавшим кодом нет — нечёткий поиск по всему словарю.
3. **Формирование результата** — `code` из найденной записи + `customDesc` (если есть) или оригинальный текст ошибки.

Нечёткий поиск: точное вхождение подстроки (case-insensitive) → скользящее окно с нормализованным расстоянием Левенштейна.

## YAML-словарь ошибок

Файл `config/errors.yaml`:

```yaml
preprocess-error:
  vocabulary:
    - key: "2001"
      substring: "unexpected symbol:"
      customDesc: "Недопустимый символ в назначении перевода"
      code: "81002"
    - key: "2002"
      substring: "Уточните у получателя"
      code: "81001"
```

| Поле | Обязательное | Описание |
|------|:---:|---------|
| `key` | да | Код ошибки исходной системы |
| `substring` | да | Подстрока для нечёткого поиска |
| `code` | да | Новый код ошибки |
| `customDesc` | нет | Кастомный текст подмены (если не задан — используется оригинальный текст) |

## Настройки

Файл `config/settings.toml`:

```toml
[input]
code_fields = ["code", "errorCode"]
message_fields = ["title", "message", "errorMessage"]

[matching]
fuzzy_threshold = 0.4

[files]
errors_yaml = "config/errors.yaml"
```

- `code_fields` — имена полей JSON, где искать код ошибки
- `message_fields` — имена полей JSON, где искать текст ошибки
- `fuzzy_threshold` — порог нечёткого поиска (0.0–1.0)

## CLI

```
error-remapper [OPTIONS] [INPUT_JSON]

Аргументы:
  [INPUT_JSON]          JSON с ошибкой (если не указан — читает из stdin)

Опции:
  -c, --config <PATH>   Путь к settings.toml [по умолчанию: config/settings.toml]
  -e, --errors <PATH>   Путь к errors.yaml (перекрывает настройки)
  -v, --verbose          Подробный вывод
  -h, --help             Справка
  -V, --version          Версия
```

## Выходной формат

```json
{"code": "81005", "customDesc": "Перевод отклонён банком получателя", "matched": true}
```

Если совпадение не найдено — `matched: false`, возвращаются оригинальные код и текст.

## Тестирование

```bash
cargo test
```

## Лицензия

MIT
