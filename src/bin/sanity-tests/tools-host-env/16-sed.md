Requirements:

- Create, update, or edit files only in `00-sanity-tests/`.

First run `mkdir -p` for that directory, then `cd` into it. Create `16-sed-source.txt` with:

```text
hello world
hello agent
hello tools
```

Use `sed` to replace every `hello` with `goodbye`, saving to `16-sed-result.txt`. Keep the source unchanged.
