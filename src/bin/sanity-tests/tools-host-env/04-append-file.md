# Test: append

First command: run `mkdir -p 00-sanity-tests`.

Create `00-sanity-tests/04-append-file.txt` with this initial content:

```text
foo
```

Then use an append operation to add:

```text
bar
```

The final file must contain exactly:

```text
foo
bar
```

Do not overwrite the initial content.
