# JSON Copy Keys

Use VS Code's built-in Regex Find and Replace:

1. Press `Ctrl+H` (`Cmd+Option+F` on Mac) to open Replace.
2. Enable Regular Expressions (`.*` icon).

## 1. Copy items only (no string wrapper or commas)

3. Find: `^\s*"([^"]+)"\s*:.*$`
4. Replace: `$1`
5. Click **Replace All** to strip values and keep only the keys.

## 2. Copy items with wrapping "" and comma deliinator

3. Find `^\s*"([^"]+)"\s*:.*$`
4. Replace: `$1`
5. Click **Replace All** to strip values and keep only the keys.

## 3. Copy items with wrapping "" and comma deliinator

```
^\s*"([^"]+)"\s*:.*$
```
