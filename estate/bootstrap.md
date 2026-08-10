# Lets do a MVP onboard/bootstrap/template flow.

- 1. Create a shared dir for IDE, toolchain, cache, daemon, lsp, executable

```rust
fn bootstrap() {
    ~/.loi/exe
          /ide
          /cache
          /db.sqlite
          /lsp
          /core
          /toolchain
          /daemon
          /dashboard
          /executable
          /settings.json
}
```

- 2. Install the src files which enable a selectable/configurable template for an immedaite showcase of paradigm concepts/constructs/powers.
  - 1 persona for now, developers.

````rust
fn init_estate() {
    - Where best to define the templates LOCALLY u think? Inside this boostrap crate?
    - Where best to source them as a client? API Call? Or inside the downloaded executable?
    - The downloaded executable? A remote API...?
    - Either way, I think obj with url/version/persona/files list makes sense.
      ```json
        {
            "url": "https://github.com/primetimetran/delta-interval-paradigm/assets",
            "version": "v0.0.1",
            "persona": "developer"
            "files": [
                {"name": "main.js", content: "", type: "js"},
                {"name": "main.rs", content: "", type: "rs"},
                {"name": "main.js", content: "", type: "js"},
                {"name": "main.loi", content: "", type: "loi"},
                {"name": "main.py", content: "", type: "py"" },
            ]
        }
    ```
}
````

- 3. Let's begin outlineing the cli commands

| name           | purpose                                                                                                                         | flags | arguments                                                          | Details                                                                                                                                                                                            |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------- | ----- | ------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| init           | Initialize a dir as an estate                                                                                                   |       | -n "estate" or '." for here. where the estate name is the dir name | Creates an estate "inode index file with uid for files" enabling aliases of dirs/files, another derivative of delta concept.                                                                       |
|                |                                                                                                                                 |       |                                                                    |                                                                                                                                                                                                    |
| doctor         | generate env for debugging and use in terms of optimizing the additional deps installed                                         |       |                                                                    |                                                                                                                                                                                                    |
| update         | update estate engine via cli                                                                                                    |       |                                                                    |                                                                                                                                                                                                    |
| version        | print version of estate engine                                                                                                  |       |                                                                    |                                                                                                                                                                                                    |
| lsp            | turn on/expose LSP server for VSCode/Zed which analyzes .loi, .md, .py, .js, .go, and .rs files                                 |       |                                                                    |                                                                                                                                                                                                    |
| add            | stub for now.                                                                                                                   |       |                                                                    |                                                                                                                                                                                                    |
| hide           | removes dir/file from current view                                                                                              |       |                                                                    |                                                                                                                                                                                                    |
| fork? diverge? | generates a new view from the current estate.                                                                                   |       | -n "compiler"                                                      | Creates a ./loi/views/compiler/settings.json which enables                                                                                                                                         |
|                |                                                                                                                                 |       |                                                                    |                                                                                                                                                                                                    |
| bundle         | creates a versioned bundle for blog/knowledge base of docs for web platform or ./docs/v[n.n.n] for their "project/estate/thing" |       |                                                                    | should write to ./public/docs/v1.0.0 the current derived projection of the "docs" from the ./src dir, the workspace. For now, match the current ./src directory example in the ./public/docs/v dir |
|                |                                                                                                                                 |       |                                                                    |                                                                                                                                                                                                    |
