Requirements:

- Create, update, or edit files only using paths under `00-sanity-tests/`; do not rely on changing directories.

Your first shell command must be `mkdir -p 00-sanity-tests`. Create `00-sanity-tests/18-grep-source.txt` with:

```text
INFO startup
ERROR failure
INFO running
WARNING something
ERROR another failure
```

Use `grep` to save lines containing `ERROR` to `00-sanity-tests/18-grep-result.txt`.
