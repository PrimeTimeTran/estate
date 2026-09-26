



find . -type d \( \
  -name node_modules -o \
  -name .venv -o \
  -name venv -o \
  -name venv -o \
  -name target -o \
  -name dist \
\) -prune -exec rm -rf {} +


