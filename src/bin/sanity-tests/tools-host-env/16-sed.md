Requirements:

- Create, update, or edit files only using paths under `00-sanity-tests/`; do not rely on changing directories.

Your first shell command must be `mkdir -p 00-sanity-tests`. Create `00-sanity-tests/16-sed-source.txt` with:

```text
hello world
hello agent
hello tools
```

Use `sed` to replace every `hello` with `goodbye`, saving to `00-sanity-tests/16-sed-result.txt`. Keep the source unchanged.
