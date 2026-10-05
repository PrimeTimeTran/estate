Requirements:

- Create, update, or edit files only using paths under `00-sanity-tests/`; do not rely on changing directories.

Your first shell command must be `mkdir -p 00-sanity-tests`. Use `find 00-sanity-tests -type f -name '*.txt' ! -path '00-sanity-tests/10-find.txt'` and save the matching paths, one per line, to `00-sanity-tests/10-find.txt`.
