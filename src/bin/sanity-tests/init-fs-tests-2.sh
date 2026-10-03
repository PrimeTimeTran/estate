#!/usr/bin/env bash
set -euo pipefail

OUT="${1:-./ai-qa}"

mkdir -p "$OUT"
rm -f "$OUT"/*.md

cat > "$OUT/01-create-file.md" <<'EOF'
Work only inside `./tests`.

Create exactly one file:

./tests/01-create-file.txt

The file must be empty.

Do not create or modify any other files.
Do not create anything outside `./tests`.
EOF

cat > "$OUT/02-create-content.md" <<'EOF'
Work only inside `./tests`.

Create exactly one file:

./tests/02-create-content.txt

Write exactly this text into it:

foobar

Do not add a newline, extra whitespace, quotes, or any other content.

Do not create or modify any other files.
Do not create anything outside `./tests`.
EOF

cat > "$OUT/03-overwrite-file.md" <<'EOF'
Work only inside `./tests`.

The file already exists:

./tests/03-overwrite-file.txt

Replace its entire contents with exactly:

foobar

Do not append to the existing contents.

Do not create or modify any other files.
Do not create anything outside `./tests`.
EOF

cat > "$OUT/04-append-file.md" <<'EOF'
Work only inside `./tests`.

The file already exists:

./tests/04-append-file.txt

It currently contains:

foobar

Append exactly one new line containing:

baz

The final file must contain exactly:

foobar
baz

Do not modify or create any other files.
Do not create anything outside `./tests`.
EOF

cat > "$OUT/05-read-file.md" <<'EOF'
Work only inside `./tests`.

Read this file:

./tests/05-read-file.txt

Do not modify it.

Report its contents exactly as they currently exist.

Do not create or modify any other files.
Do not create anything outside `./tests`.
EOF

cat > "$OUT/06-rename-file.md" <<'EOF'
Work only inside `./tests`.

Create this file:

./tests/06-rename-file.txt

Write exactly:

rename me

Then rename it to:

./tests/06-renamed-file.txt

Preserve its contents exactly.

Do not create a copy.

Do not create or modify any other files.
Do not create anything outside `./tests`.
EOF

cat > "$OUT/07-delete-file.md" <<'EOF'
Work only inside `./tests`.

Create this file:

./tests/07-delete-file.txt

Write exactly:

delete me

Then delete the file.

The final state must be that:

./tests/07-delete-file.txt

does not exist.

Do not create or modify any other files.
Do not create anything outside `./tests`.
EOF

cat > "$OUT/08-create-directory.md" <<'EOF'
Work only inside `./tests`.

Create this directory:

./tests/08-create-directory/

Inside it create:

./tests/08-create-directory/08-create-directory.txt

The file must contain exactly:

hello

Do not create or modify anything outside `./tests`.
EOF

cat > "$OUT/09-multiple-files.md" <<'EOF'
Work only inside `./tests`.

Create this directory:

./tests/09-multiple-files/

Inside it create exactly these three files:

./tests/09-multiple-files/a.txt
./tests/09-multiple-files/b.txt
./tests/09-multiple-files/c.txt

Their contents must be exactly:

a.txt → A
b.txt → B
c.txt → C

Also create this marker file:

./tests/09-multiple-files.txt

The marker file must contain exactly:

multiple files

Do not create any other files or directories.

Do not modify anything outside `./tests`.
EOF

cat > "$OUT/10-crud-sequence.md" <<'EOF'
Work only inside `./tests`.

Perform the following sequence exactly inside `./tests`:

1. Create:

./tests/10-crud-sequence-todo.txt

2. Write exactly:

buy milk

3. Rename it to:

./tests/10-crud-sequence-tasks.txt

4. Replace its contents with exactly:

buy milk
buy eggs

5. Append exactly one new line:

buy bread

6. Read the final file.

The final contents of:

./tests/10-crud-sequence-tasks.txt

must be exactly:

buy milk
buy eggs
buy bread

There must be no:

./tests/10-crud-sequence-todo.txt

Also create:

./tests/10-crud-sequence.txt

containing exactly:

CRUD sequence completed

Do not create any other files.
Do not create anything outside `./tests`.
EOF

cat > "$OUT/11-constrained-crud.md" <<'EOF'
Work only inside `./tests`.

Create this directory:

./tests/11-constrained-crud/

Inside it:

1. Create:

./tests/11-constrained-crud/one.txt

containing exactly:

one

2. Create:

./tests/11-constrained-crud/two.txt

containing exactly:

two

3. Rename `one.txt` to:

./tests/11-constrained-crud/first.txt

4. Replace the contents of `first.txt` with exactly:

first

Do not modify `two.txt`.

Then create:

./tests/11-constrained-crud.txt

containing exactly:

constrained CRUD completed

Do not create any additional files.

Do not create anything outside `./tests`.
EOF

cat > "$OUT/12-transform-content.md" <<'EOF'
Work only inside `./tests`.

Create exactly one file:

./tests/12-transform-content.txt

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
Do not create anything outside `./tests`.
EOF

cat > "$OUT/13-multi-directory.md" <<'EOF'
Work only inside `./tests`.

Create the following directory structure:

./tests/13-multi-directory/
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

Also create:

./tests/13-multi-directory.txt

containing exactly:

multi-directory completed

Create exactly these directories and files.

Do not create anything outside `./tests`.
EOF

cat > "$OUT/14-alphabet-files.md" <<'EOF'
Work only inside `./tests`.

Create this directory:

./tests/14-alphabet-files/

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

Also create:

./tests/14-alphabet-files.txt

containing exactly:

alphabet files completed

Do not create or modify anything outside `./tests`.
EOF

cat > "$OUT/15-alphabet-python.md" <<'EOF'
Work only inside `./tests`.

Create this directory:

./tests/15-alphabet-python/

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

Also create:

./tests/15-alphabet-python.txt

containing exactly:

alphabet python files completed

Do not create or modify anything outside `./tests`.
EOF

cat > "$OUT/16-alphabet-sequential-dates.md" <<'EOF'
Work only inside `./tests`.

Create this directory:

./tests/16-alphabet-sequential-dates/

For every letter of the English alphabet, create exactly one `.py` file named after that lowercase letter.

Use the YYYY-MM-DD date format.

The file:

./tests/16-alphabet-sequential-dates/a.py

must contain today's date.

The file:

./tests/16-alphabet-sequential-dates/b.py

must contain the date one day after today's date.

The file:

./tests/16-alphabet-sequential-dates/c.py

must contain the date two days after today's date.

Continue this pattern through `z.py`, incrementing the date by exactly one day for every subsequent letter.

For example, if today is 2026-10-03:

a.py → 2026-10-03
b.py → 2026-10-04
c.py → 2026-10-05

Continue through z.py.

Create exactly 26 `.py` files.

Also create:

./tests/16-alphabet-sequential-dates.txt

containing exactly:

alphabet sequential dates completed

Do not create or modify anything outside `./tests`.
EOF

cat > "$OUT/17-complex-project.md" <<'EOF'
Work only inside `./tests`.

Create the following project structure:

./tests/17-complex-project/
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

Also create:

./tests/17-complex-project.txt

containing exactly:

complex project completed

Create exactly the files and directories specified above.

Do not create any additional files.

Do not modify anything outside `./tests`.
EOF

echo "Generated $(find "$OUT" -name '*.md' | wc -l | tr -d ' ') QA prompts in:"
echo "  $(cd "$OUT" && pwd)"
echo
find "$OUT" -maxdepth 1 -type f -name '*.md' -print | sort