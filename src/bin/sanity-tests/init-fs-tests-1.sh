#!/usr/bin/env bash
set -euo pipefail

OUT="${1:-./ai-qa}"

mkdir -p "$OUT"
rm -f "$OUT"/*.md

cat > "$OUT/01-create-file.md" <<'EOF'
Create a file named `hello.txt` in the current project directory.

The file must be empty.

Do not create or modify any other files.
EOF

cat > "$OUT/02-create-content.md" <<'EOF'
Create a file named `hello-world.txt` in the current project directory.

Write exactly this text into it:

foobar

Do not add a newline, extra whitespace, quotes, or any other content.

Do not create or modify any other files.
EOF

cat > "$OUT/03-overwrite-file.md" <<'EOF'
The file `hello-world.txt` already exists.

Replace its entire contents with exactly:

foobar

Do not append to the existing contents.

Do not create or modify any other files.
EOF

cat > "$OUT/04-append.md" <<'EOF'
The file `hello-world.txt` already contains:

foobar

Append exactly one new line containing:

baz

The final file must contain exactly:

foobar
baz

Do not modify or create any other files.
EOF

cat > "$OUT/05-read-file.md" <<'EOF'
Read the file `hello-world.txt`.

Do not modify it.

Report its contents exactly as they currently exist.
EOF

cat > "$OUT/06-rename-file.md" <<'EOF'
Rename the file:

hello-world.txt

to:

renamed.txt

Preserve its contents exactly.

Do not create a copy.

Do not create or modify any other files.
EOF

cat > "$OUT/07-delete-file.md" <<'EOF'
Delete the file `renamed.txt`.

Do not create or modify any other files.
EOF

cat > "$OUT/08-create-directory.md" <<'EOF'
Create a directory named:

hello-project

Inside that directory create:

hello.txt

The file must contain exactly:

hello

Do not create or modify anything outside `hello-project`.
EOF

cat > "$OUT/09-multiple-files.md" <<'EOF'
Create a directory named `letters`.

Inside it create exactly these three files:

a.txt
b.txt
c.txt

Their contents must be exactly:

a.txt → A
b.txt → B
c.txt → C

Do not create any other files or directories.

Do not modify anything outside `letters`.
EOF

cat > "$OUT/10-crud-sequence.md" <<'EOF'
Perform the following sequence exactly.

1. Create `todo.txt`.
2. Write exactly `buy milk` into it.
3. Rename it to `tasks.txt`.
4. Replace its contents with exactly:

buy milk
buy eggs

5. Append exactly one new line:

buy bread

6. Read the final file.

The final contents of `tasks.txt` must be exactly:

buy milk
buy eggs
buy bread

There must be no `todo.txt`.

Do not create any other files.
EOF

cat > "$OUT/11-constrained-crud.md" <<'EOF'
Create a directory named `project`.

Inside it:

1. Create `one.txt` containing exactly:
one

2. Create `two.txt` containing exactly:
two

3. Rename `one.txt` to `first.txt`.

4. Replace the contents of `first.txt` with exactly:
first

Do not modify `two.txt`.

Do not create any additional files.

Do not create anything outside the `project` directory.
EOF

cat > "$OUT/12-transform-content.md" <<'EOF'
Create a file named `numbers.txt`.

Write the numbers 1 through 10 into the file, one number per line, in ascending order.

The final contents must be exactly:

1
2
3
4
5
6
7
8
9
10

Do not create or modify any other files.
EOF

cat > "$OUT/13-multi-directory.md" <<'EOF'
Create the following directory structure:

project/
├── src/
│   ├── main.txt
│   └── lib.txt
└── tests/
    └── test.txt

Contents:

src/main.txt:
main

src/lib.txt:
lib

tests/test.txt:
test

Create exactly these directories and files.

Do not create anything else.
EOF

cat > "$OUT/14-alphabet-files.md" <<'EOF'
Create a directory named `alphabet-test`.

Inside it create exactly one file for every letter of the English alphabet:

a.txt
b.txt
c.txt
d.txt
e.txt
f.txt
g.txt
h.txt
i.txt
j.txt
k.txt
l.txt
m.txt
n.txt
o.txt
p.txt
q.txt
r.txt
s.txt
t.txt
u.txt
v.txt
w.txt
x.txt
y.txt
z.txt

Each file must contain exactly its corresponding lowercase letter.

For example:

a.txt → a
b.txt → b
c.txt → c

Do not create or modify anything outside `alphabet-test`.
EOF

cat > "$OUT/15-alphabet-python.md" <<'EOF'
Create a directory named `alphabet-test`.

For every letter of the English alphabet, create exactly one file named after that lowercase letter using the `.py` extension.

The resulting files must be:

a.py
b.py
c.py
d.py
e.py
f.py
g.py
h.py
i.py
j.py
k.py
l.py
m.py
n.py
o.py
p.py
q.py
r.py
s.py
t.py
u.py
v.py
w.py
x.py
y.py
z.py

Each file must contain exactly its corresponding lowercase letter.

For example:

a.py → a
b.py → b
c.py → c

Do not create or modify anything outside `alphabet-test`.
EOF

cat > "$OUT/16-alphabet-sequential-dates.md" <<'EOF'
Create a directory named `alphabet-test`.

For every letter of the English alphabet, create exactly one `.py` file named after that lowercase letter.

Use the YYYY-MM-DD date format.

The file `a.py` must contain today's date.

The file `b.py` must contain the date one day after today's date.

The file `c.py` must contain the date two days after today's date.

Continue this pattern through `z.py`, incrementing the date by exactly one day for every subsequent letter.

For example, if today is 2026-10-03:

a.py → 2026-10-03
b.py → 2026-10-04
c.py → 2026-10-05

Continue through z.py.

Create exactly 26 files.

Do not create or modify anything outside `alphabet-test`.
EOF

cat > "$OUT/17-complex-project.md" <<'EOF'
Create the following project structure:

qa-project/
├── README.md
├── src/
│   ├── main.py
│   ├── config.py
│   └── utils.py
├── tests/
│   ├── test_main.py
│   └── test_utils.py
└── data/
    ├── input.txt
    └── output.txt

Write exactly the following contents.

README.md:

# QA Project

This project was created by the instruction-following test.

src/main.py:

print("hello")

src/config.py:

NAME = "qa"

src/utils.py:

def add(a, b):
    return a + b

tests/test_main.py:

def test_main():
    assert True

tests/test_utils.py:

def test_add():
    assert 1 + 1 == 2

data/input.txt:

input

data/output.txt:

output

Create exactly the files and directories specified above.

Do not create any additional files.

Do not modify anything outside `qa-project`.
EOF

echo "Generated $(find "$OUT" -name '*.md' | wc -l | tr -d ' ') QA prompts in:"
echo "  $(cd "$OUT" && pwd)"
echo
find "$OUT" -maxdepth 1 -type f -name '*.md' -print | sort