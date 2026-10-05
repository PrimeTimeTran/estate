Requirements:
- Create, update, or edit files only in `00-sanity-tests/`.

First run `mkdir -p` for that directory, then `cd` into it. Create `18-grep-source.txt` with:

```text
INFO startup
ERROR failure
INFO running
WARNING something
ERROR another failure
```

Use `grep` to save lines containing `ERROR` to `18-grep-result.txt`.
