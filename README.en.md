# error-remapper

[![ru](https://img.shields.io/badge/lang-Русский-green)](README.md)

A Rust CLI utility for fuzzy matching and remapping backend error codes using a YAML dictionary.

## Why

External backend systems return errors in various formats with different codes and messages. This utility accepts a JSON error, finds the best matching entry in a YAML dictionary (by code and/or fuzzy text matching), and returns a unified error code and description.

## Installation

```bash
git clone https://github.com/<your-username>/error-remapper.git
cd error-remapper
cargo build --release
```

Binary: `target/release/error-remapper`

## Quick Start

```bash
# Pass JSON as an argument
error-remapper '{"error":{"code":"3011","title":"Не пройден фрод-мониторинг"}}'

# Result:
# {"code":"81005","customDesc":"Перевод отклонён банком получателя","matched":true}

# Pass JSON via stdin
echo '{"error":{"code":"2001","title":"Got unexpected symbol: @"}}' | error-remapper
```

## Matching Algorithm

1. **Exact code match** — if the error code from JSON matches a `key` in YAML and exactly one entry is found, it is used.
2. **Fuzzy text matching** — if no exact match (0 or multiple entries with the same `key`):
   - Multiple entries with matching code → fuzzy search only among them.
   - No entries with matching code → fuzzy search across the entire dictionary.
3. **Result construction** — `code` from the matched entry + `customDesc` (if present) or the original error text.

Fuzzy matching: exact case-insensitive substring containment → sliding window with normalized Levenshtein distance.

## YAML Error Dictionary

File `config/errors.yaml`:

```yaml
preprocess-error:
  vocabulary:
    - key: "2001"
      substring: "unexpected symbol:"
      customDesc: "Invalid character in transfer destination"
      code: "81002"
    - key: "2002"
      substring: "Check with recipient"
      code: "81001"
```

| Field | Required | Description |
|-------|:---:|------------|
| `key` | yes | Source system error code |
| `substring` | yes | Substring for fuzzy matching |
| `code` | yes | Remapped error code |
| `customDesc` | no | Custom replacement text (if absent — original text is used) |

## Settings

File `config/settings.toml`:

```toml
[input]
code_fields = ["code", "errorCode", "statusCode"]
message_fields = ["title", "message", "errorMessage", "errorText"]

[matching]
fuzzy_threshold = 0.4

[output]
pretty = false

# Output JSON template: key = field name, value = expression with placeholders
[output.template]
statusCode = "{{code}}"
errorText = "{{description}}"
errorDescription = "{{input.ErrorDescription}}"

[files]
errors_yaml = "config/errors.yaml"
```

- `code_fields` — JSON field names to search for the error code
- `message_fields` — JSON field names to search for the error message
- `fuzzy_threshold` — fuzzy matching threshold (0.0–1.0)
- `output.template` — output JSON template (see below)

### Output template placeholders

| Placeholder | Description |
|---|---|
| `{{code}}` | Remapped error code |
| `{{description}}` | Remapped description |
| `{{matched}}` | Whether a match was found (true/false) |
| `{{original_code}}` | Original code from input JSON |
| `{{original_message}}` | Original message from input JSON |
| `{{input.FIELD}}` | Any field from input JSON (supports nesting: `input.error.detail`) |

## CLI

```
error-remapper [OPTIONS] [INPUT_JSON]

Arguments:
  [INPUT_JSON]          Error JSON string (reads from stdin if omitted)

Options:
  -c, --config <PATH>   Path to settings.toml [default: config/settings.toml]
  -e, --errors <PATH>   Path to errors.yaml (overrides settings)
  -v, --verbose          Verbose output
  -h, --help             Help
  -V, --version          Version
```

## Output Format

```json
{"code": "81005", "customDesc": "Transfer rejected by recipient bank", "matched": true}
```

If no match is found — `matched: false`, original code and text are returned.

## Java Integration (JNA)

The utility compiles as a shared library (`.dylib` / `.so`), allowing it to be called from Java via JNA.

### Building the shared library

```bash
cargo build --release
# Result: target/release/liberror_remapper.dylib (macOS) / .so (Linux)
```

### Maven dependency

```xml
<dependency>
    <groupId>net.java.dev.jna</groupId>
    <artifactId>jna</artifactId>
    <version>5.16.0</version>
</dependency>
```

### Usage

```java
import com.thothlab.remapper.ErrorRemapper;

ErrorRemapper remapper = new ErrorRemapper("/path/to/config");
String result = remapper.remap("{\"statusCode\": \"3011\", \"errorText\": \"Не пройден фрод\"}");
System.out.println(result);
```

### Running

```bash
java -Djna.library.path=/path/to/target/release \
     -cp target/classes:jna-5.16.0.jar \
     com.thothlab.remapper.Example config
```

Java wrapper is located in the `java/` directory:

```
java/
├── pom.xml
└── src/main/java/com/thothlab/remapper/
    ├── ErrorRemapper.java   # JNA wrapper
    └── Example.java         # Usage example
```

## Testing

```bash
cargo test
```

## License

MIT
