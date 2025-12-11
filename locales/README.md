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

### 2. Update Comments

Update the header comment to document which dialect this base represents:

```ftl
# ===========================================
#      French (fr) — Base Language File
# ===========================================
# This is the base translation file for French, using Metropolitan French (fr_FR).
# Regional variants (e.g., fr_CA.ftl) should only override keys that differ.
```

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
```ftl
book-label = { $count ->
    [one] Book
    [other] Books
}
```

### Variables
```ftl
page-number = Page { $count }
yearly-summary = Yearly Summary { $count }
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
