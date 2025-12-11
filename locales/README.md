# Translations

This directory contains Fluent (`.ftl`) translation files for KoShelf.

## Locale Hierarchy

Translations follow a three-tier fallback system:

```
Regional Variant (e.g., de_AT.ftl)  ← sparse, only overrides
        ↓
Base Language (e.g., de.ftl)       ← complete translations
        ↓
English Fallback (en.ftl)          ← ultimate fallback
```

**Key principle**: The base language file is whichever regional dialect is added first.

For example:
- If Brazilian Portuguese (`pt_BR`) is contributed first, `pt.ftl` uses Brazilian Portuguese
- A later `pt_PT.ftl` would contain only European Portuguese overrides
- This is the opposite of what you might expect, but reflects real contribution order

## Adding a New Language

### 1. Create the Base Language File

Copy `en.ftl` as your starting point:

```bash
cp en.ftl <lang>.ftl   # e.g., cp en.ftl fr.ftl
```

Translate all keys. The base file must be **complete**—every key from `en.ftl` should exist.

### 2. Add Required Metadata

Every base language file **must** include machine-readable metadata (used by `--list-languages`):

```ftl
# ===========================================
#      French (fr) — Base Language File
# ===========================================
# This is the base translation file for French, using Metropolitan French (fr_FR).
# Regional variants (e.g., fr_CA.ftl) should only override keys that differ.

# Machine-readable metadata (used by --list-languages)
-lang-code = fr
-lang-name = Français    # Native language name
-lang-dialect = fr_FR
```

| Key | Purpose |
|-----|---------|
| `-lang-code` | ISO 639-1 base language code (e.g., `fr`, `de`) |
| `-lang-name` | Native name of the language |
| `-lang-dialect` | Full locale code for date formatting (e.g., `fr_FR`, `de_AT`) |

### 3. Adding Regional Variants (Optional)

Regional variant files are **sparse**—they only contain keys that differ from the base:

```ftl
# ===========================================
#      French Canadian (fr_CA) — Regional Variant
# ===========================================
# Only overrides that differ from fr.ftl (Metropolitan French)

# Canadian French uses different terms for some UI elements
filter-completed = Terminé
```

## File Naming Convention

| File | Purpose |
|------|---------|
| `en.ftl` | English base (required, ultimate fallback) |
| `de.ftl` | German base |
| `de_AT.ftl` | Austrian German overrides |
| `pt.ftl` | Portuguese base (whichever dialect first) |
| `pt_BR.ftl` | Brazilian overrides (if `pt.ftl` is European) |

## FTL Syntax Reference

### Simple Keys
```ftl
books = Books
loading = Loading...
```

### Pluralization
KoShelf supports the full set of [Unicode CLDR plural categories](http://cldr.unicode.org/index/cldr-spec/plural-rules): `zero`, `one`, `two`, `few`, `many`, and `other`.

```ftl
book-label = { $count ->
    [one] Book
   *[other] Books
}

items-found = { $count ->
    [zero] No items found
    [one] One item found
    [few] A few items found
   *[other] { $count } items found
}
```

**Important**: The categories (`zero`, `one`, `two`, `few`, `many`, `other`) follow strict [CLDR Plural Rules](https://cldr.unicode.org/index/cldr-spec/plural-rules) for each language. 
- You do NOT need to define all categories. Only define those used by your language.
- `[zero]` refers to the *grammatical* category "zero" (present in languages like Latvian or Arabic), NOT necessarily the number 0. 
- In many languages (like English), the number 0 falls into `[other]` or `[one]`, so you would handle it there.
- The `*` prefix marks the **default variant** (e.g., `*[other]`). You **MUST** have exactly one default variant per plural block. It is standard practice to attach this to the `[other]` category, as that catches everything not covered by specific rules.

### Variables
KoShelf's parser allows variables, but with strict limitations for pluralized/numeric strings:
- **Only `$count` is supported** for automatic number formatting.
- **Strict formatting**: You must use `{$count}` or `{ $count }`.

```ftl
page-number = Page { $count }
yearly-summary = Yearly Summary { $count }
```

### Multiline Messages
Long messages can be split across multiple lines. Continuation lines **must be indented**:

```ftl
introduction =
    Welcome to KoShelf!
    This is a long message that spans
    multiple lines.
```

## Testing

Run the i18n tests to verify your translations:

```bash
cargo test i18n
```

The test suite checks:
- All keys in non-English files exist in `en.ftl`
- FTL syntax is valid
- Pluralization works correctly
