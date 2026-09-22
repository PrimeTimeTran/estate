It is true that the vscode-icons engine treats files and folders as completely separate buckets. You cannot natively cross-reference a built-in file icon name inside a folder configuration block. Because vscode-icons does not have a built-in Rust folder asset, pointing to "icon": "rust" inside your folder configuration maps to a non-existent asset, resulting in a blank or fallback folder icon. [1]
However, you can work around this limitation by manually downloading a Rust SVG icon and registering it as a custom folder icon. [2]
------------------------------

## Step 1: Create Your Custom Icon Directory

You need to put your custom graphics into a specialized folder so the extension knows where to find them: [3]

1.  Open your operating system's file manager and locate your global VS Code User configurations directory:

- Windows: %APPDATA%\Code\User\
  - Mac: ~/Library/Application Support/Code/User/
  - Linux: ~/.config/Code/User/ [3]

2.  Inside that User directory, create a folder named exactly vsicons-custom-icons. [3]

## Step 2: Save the Rust Folders (Closed and Opened variants)

Because directories animate when clicked, vscode-icons expects two unique graphics for every folder association. [2, 4]

1.  Grab a clean Rust asset logo in SVG format (you can download one safely from online asset platforms).
2.  Save two copies of it inside your brand-new vsicons-custom-icons folder using these exact names:

- folder_type_rustacean.svg (This displays when closed)
  - folder_type_rustacean_opened.svg (This displays when opened) [2, 4]

## Step 3: Map It Cleanly in settings.json

Now that your local files are sitting in the target system directory, add this configuration block to your global settings file: [2, 4]

"vsicons.associations.folders": [
{
"icon": "rustacean",
"extensions": ["rustacean"],
"filename": true,
"format": "svg"
}
]

(Notice how we dropped the folder_type_ prefix here. The extension handles the prefix attachment logic automatically behind the scenes). [2, 4]

## Step 4: Rebuild & Apply Changes

Force an index refresh to register your custom file insertions: [5]

1.  Press Ctrl+Shift+P (or Cmd+Shift+P on macOS) to bring up the Command Palette.
2.  Run VSCode Icons: Apply Icons Customization. [5]

Your editor workspace tree will instantly update with your custom folder assets!
If you run into issues finding or saving the SVGs, let me know:

-
- What operating system are you running on?
- Would you like me to map multiple folder names to this same custom icon asset?
-

[1] [https://github.com](https://github.com/vscode-icons/vscode-icons/wiki/FineTuning)
[2] [https://github.com](https://github.com/vscode-icons/vscode-icons/wiki/Custom)
[3] [https://github.com](https://github.com/vscode-icons/vscode-icons/wiki/Custom)
[4] [https://stackoverflow.com](https://stackoverflow.com/questions/72512326/vscode-icons-extension-cannot-use-custom-icon)
[5] [https://marketplace.visualstudio.com](https://marketplace.visualstudio.com/items?itemName=vscode-icons-team.vscode-icons)
