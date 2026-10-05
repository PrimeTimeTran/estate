# Test: shell pipeline

First command: run `mkdir -p 00-sanity-tests`.

Create `00-sanity-tests/25-pipeline-source.txt` with:

```text
apple
banana
apple
cherry
banana
apple
```

Use a shell pipeline with standard CLI tools to sort the lines, count identical adjacent values, and output one line per fruit with its count.

Save the result to `00-sanity-tests/25-pipeline-result.txt`. It must be exactly:

```text
3 apple
2 banana
1 cherry
```

Use actual CLI tools; do not calculate the answer manually.
