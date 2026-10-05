Requirements:

- Create, update, or edit files only using paths under `00-sanity-tests/`; do not rely on changing directories.

Your first shell command must be `mkdir -p 00-sanity-tests`. Create `00-sanity-tests/04-append.txt` with:

```text
foo
```

Then append:

```text
bar
```

The final file must contain exactly:

```text
foo
bar
```

Do not overwrite the initial content.
