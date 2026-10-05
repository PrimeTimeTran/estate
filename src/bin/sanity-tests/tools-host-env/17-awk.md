Requirements:

- Create, update, or edit files only using paths under `00-sanity-tests/`; do not rely on changing directories.

Your first shell command must be `mkdir -p 00-sanity-tests`. Create `00-sanity-tests/17-awk-source.txt` with:

```text
apple 10
banana 20
cherry 30
```

Use `awk` to extract the fruit names, one per line, to `00-sanity-tests/17-awk-result.txt`.
