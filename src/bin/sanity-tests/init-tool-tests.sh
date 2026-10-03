#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:-agent-tool-tests}"

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

cat > "$OUT_DIR/README.md" <<'EOF'
# External CLI Tool Capability Tests

Each `.md` file is an independent prompt for the agent.

These tests verify that the agent can actually invoke external CLI
capabilities rather than merely describe or simulate what those tools
would do.

Run individual tests or the complete corpus from the test directory.

Tests:

01-mkdir
02-touch
03-write-file
04-append-file
05-cat
06-cp
07-mv
08-rm
09-ls
10-find
11-rg
12-head
13-tail
14-sort
15-wc
16-sed
17-awk
18-grep
19-git-status
20-git-diff
21-git-log
22-git-show
23-curl
24-env
25-shell-pipeline
EOF

cat > "$OUT_DIR/01-mkdir.md" <<'EOF'
# Test: mkdir

Use the `mkdir` CLI tool.

Create a directory named:

`01-created-directory`

Do not create any other files or directories.
EOF

cat > "$OUT_DIR/02-touch.md" <<'EOF'
# Test: touch

Use the `touch` CLI tool.

Create an empty file named:

`02-touched-file.txt`

Do not write any content to the file.
EOF

cat > "$OUT_DIR/03-write-file.md" <<'EOF'
# Test: write file

Create a file named:

`03-write-file.txt`

with exactly this content:

```text
hello
world
foobar
````

Do not add any additional lines or text.
EOF

cat > "$OUT_DIR/04-append-file.md" <<'EOF'

# Test: append

Create a file named:

`04-append-file.txt`

with this initial content:

```text
foobar
```

Then append this content to the same file:

```text
baz
```

The final file must contain exactly:

```text
foobar
baz
```

Do not overwrite the initial content when performing the append.
EOF

cat > "$OUT_DIR/05-cat.md" <<'EOF'

# Test: cat

Use the `cat` CLI tool.

Read:

`03-write-file.txt`

Then create:

`05-cat-result.txt`

containing exactly the contents read from `03-write-file.txt`.

Do not use the original file as the output file.
EOF

cat > "$OUT_DIR/06-cp.md" <<'EOF'

# Test: cp

Use the `cp` CLI tool.

Copy:

`03-write-file.txt`

to:

`06-copied-file.txt`

The copied file must have identical contents.
EOF

cat > "$OUT_DIR/07-mv.md" <<'EOF'

# Test: mv

Use the `mv` CLI tool.

Rename:

`02-touched-file.txt`

to:

`07-moved-file.txt`

Do not create a second copy.

The original filename should no longer exist.
EOF

cat > "$OUT_DIR/08-rm.md" <<'EOF'

# Test: rm

Use the `rm` CLI tool.

Delete:

`07-moved-file.txt`

Do not delete any other file.
EOF

cat > "$OUT_DIR/09-ls.md" <<'EOF'

# Test: ls

Use the `ls` CLI tool.

List the files in the current directory.

Write the resulting listing to:

`09-ls-result.txt`

Do not modify any other files.
EOF

cat > "$OUT_DIR/10-find.md" <<'EOF'

# Test: find

Use the `find` CLI tool.

Find every `.txt` file inside the current test directory.

Write the matching paths to:

`10-find-result.txt`

Use one path per line.

Do not modify or delete the files you find.
EOF

cat > "$OUT_DIR/11-rg.md" <<'EOF'

# Test: ripgrep

Use `rg` (ripgrep).

Search all files in the current test directory for:

`foobar`

Write the matching results to:

`11-rg-result.txt`

Preserve the normal `rg` output format.

Do not modify the files being searched.
EOF

cat > "$OUT_DIR/12-head.md" <<'EOF'

# Test: head

Create:

`12-head-source.txt`

with exactly:

```text
one
two
three
four
five
```

Then use the `head` CLI tool to extract only the first two lines.

Write the result to:

`12-head-result.txt`
EOF

cat > "$OUT_DIR/13-tail.md" <<'EOF'

# Test: tail

Create:

`13-tail-source.txt`

with exactly:

```text
one
two
three
four
five
```

Then use the `tail` CLI tool to extract only the last two lines.

Write the result to:

`13-tail-result.txt`
EOF

cat > "$OUT_DIR/14-sort.md" <<'EOF'

# Test: sort

Create:

`14-sort-source.txt`

with exactly:

```text
delta
alpha
charlie
bravo
```

Use the `sort` CLI tool to sort the lines alphabetically.

Write the result to:

`14-sort-result.txt`

The result must be:

```text
alpha
bravo
charlie
delta
```

EOF

cat > "$OUT_DIR/15-wc.md" <<'EOF'

# Test: wc

Use the `wc` CLI tool.

Run `wc` against:

`03-write-file.txt`

Write the command's output to:

`15-wc-result.txt`

Do not manually calculate the result. The result must come from `wc`.
EOF

cat > "$OUT_DIR/16-sed.md" <<'EOF'

# Test: sed

Create:

`16-sed-source.txt`

with:

```text
hello world
hello agent
hello tools
```

Use `sed` to replace every occurrence of:

`hello`

with:

`goodbye`

Write the transformed output to:

`16-sed-result.txt`

Do not modify the source file.
EOF

cat > "$OUT_DIR/17-awk.md" <<'EOF'

# Test: awk

Create:

`17-awk-source.txt`

with:

```text
apple 10
banana 20
cherry 30
```

Use `awk` to extract only the fruit names.

Write the result to:

`17-awk-result.txt`

The result should contain one fruit per line.
EOF

cat > "$OUT_DIR/18-grep.md" <<'EOF'

# Test: grep

Create:

`18-grep-source.txt`

with:

```text
INFO startup
ERROR failure
INFO running
WARNING something
ERROR another failure
```

Use `grep` to find only lines containing:

`ERROR`

Write the result to:

`18-grep-result.txt`
EOF

cat > "$OUT_DIR/19-git-status.md" <<'EOF'

# Test: git status

Use `git status`.

Do not modify the repository.

Run:

```bash
git status --short
```

Write the output to:

`19-git-status-result.txt`
EOF

cat > "$OUT_DIR/20-git-diff.md" <<'EOF'

# Test: git diff

Use `git diff`.

Do not modify the repository.

Run:

```bash
git diff
```

Write the output to:

`20-git-diff-result.txt`
EOF

cat > "$OUT_DIR/21-git-log.md" <<'EOF'

# Test: git log

Use `git log`.

Do not modify the repository.

Run:

```bash
git log --oneline -5
```

Write the output to:

`21-git-log-result.txt`
EOF

cat > "$OUT_DIR/22-git-show.md" <<'EOF'

# Test: git show

Use `git show`.

Do not modify the repository.

Run:

```bash
git show --stat --oneline HEAD
```

Write the output to:

`22-git-show-result.txt`
EOF

cat > "$OUT_DIR/23-curl.md" <<'EOF'

# Test: curl

Use `curl`.

Make an HTTP GET request to:

`https://example.com`

Save the response body to:

`23-curl-result.html`

Do not use a browser.

Do not modify any files other than `23-curl-result.html`.
EOF

cat > "$OUT_DIR/24-env.md" <<'EOF'

# Test: env

Use the `env` CLI command.

Capture the current process environment.

Write the output to:

`24-env-result.txt`

Do not modify environment variables.
EOF

cat > "$OUT_DIR/25-shell-pipeline.md" <<'EOF'

# Test: shell pipeline

Create:

`25-pipeline-source.txt`

with:

```text
apple
banana
apple
cherry
banana
apple
```

Use a shell pipeline involving standard CLI tools to:

1. Sort the lines.
2. Count identical adjacent values.
3. Produce one line per fruit with its count.

Write the final result to:

`25-pipeline-result.txt`

The expected values are:

```text
3 apple
2 banana
1 cherry
```

Use actual CLI tools rather than calculating the answer manually.
EOF

echo
echo "Created external CLI capability corpus:"
echo
find "$OUT_DIR" -maxdepth 1 -type f -name '*.md' -print | sort
echo
echo "Corpus: $OUT_DIR"

