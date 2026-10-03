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
