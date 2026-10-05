# Test: grep

First command: run `mkdir -p 00-sanity-tests`.

Create `00-sanity-tests/18-grep-source.txt` with:

```text
INFO startup
ERROR failure
INFO running
WARNING something
ERROR another failure
```

Use `grep` to find only lines containing `ERROR` and save them to `00-sanity-tests/18-grep-result.txt`.
