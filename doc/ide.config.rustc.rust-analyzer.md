# Task

## Steps

1. Watch the following file for changes
   `./Cargo.toml`

2. If changed. Check the value of this key, `default`.

   ```toml
   [features]
   # default = ["native"]
   # default = ["web"]
   default = []
   ```

3. If changed, update this file, `~/Library/Application Support/Code/User/settings.json`

   ```json
   {
   	"rust-analyzer.cargo.features": ["native"]
   }
   ```

4. Also update this file, `/Users/future/.config/zed/settings.json`

```json
{
	// Update this key to match.
	"lsp": {
		"rust-analyzer": {
			"initialization_options": {
				"cargo": {
					// Update this key to match.
					"features": ["native"]
				}
			}
		}
	}
}
```
