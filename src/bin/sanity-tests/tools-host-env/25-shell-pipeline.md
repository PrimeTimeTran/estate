Requirements:

- Create, update, or edit files only in `00-sanity-tests/`.

First run `mkdir -p` for that directory, then `cd` into it. Create `25-pipeline-source.txt` with:

```text
apple
banana
apple
cherry
banana
apple
```

Use a CLI pipeline to sort and count identical lines, saving `count fruit` rows to `25-pipeline-result.txt`. Expected:

```text
3 apple
2 banana
1 cherry
```
