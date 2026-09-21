To define a new naming convention and layer your custom icons directly on top of the vscode-icons extension, you need to place your physical SVG assets in a specific custom directory and map them using the arrays in your VS Code settings. [1, 2]  
Step 1: Create Your Custom Icons Folder
The extension searches for a strict directory structure named .

1. Locate or Define Your Folder Path:By default, you can create a folder named inside your global VS Code User directory. However, a more maintainable approach is to create this folder anywhere you like (such as a shared team folder or inside your profile) and tell the extension where to find it.
2. Configure the Path in Settings:Open your and add the setting pointing to the parent folder that contains your directory (do not include the directory name itself in the path):
3. (In this example, your physical SVGs must live inside ). [3]

Step 2: Add Your Custom SVG Icons
Drop your new custom icons into that folder.

• Ensure they are vector files () or images.
• Give them distinctive filenames (e.g., ). [2, 4]

Step 3: Define Your Convention in  
To establish your new convention, map file extensions, partial match blobs, or full filenames to your custom icons using the or settings blocks. [2, 5]  
Open your global User Settings (JSON) and append your rules: [2]  
Schema Breakdown: • : The filename of your image inside without its file extension.
• : The suffix pattern or filename to trigger the icon match.
• : The file type extension of your asset (usually or ).
• : Set to if you are targeting an exact, specific filename (like or ) rather than a general file suffix extension. [2]

Step 4: Apply and Refresh the Assets
The extension does not automatically watch this custom folder in real-time, so you must force it to regenerate its internal configuration manifest.

1. Open the VS Code Command Palette ( on Windows/Linux or on Mac).
2. Type and run the command: .
3. VS Code will prompt you to restart or reload the window. Once reloaded, your custom conventions will seamlessly render alongside the standard set! [6]

To get started, tell me:

• What specific naming pattern or file extensions make up your new convention?
• Are you looking to share this custom setup across a development team repository?

AI responses may include mistakes.

[1] https://github.com/vscode-icons/vscode-icons/wiki/Custom
[2] https://medium.com/@chih.hsi.chen/how-to-add-your-custom-icons-to-vscode-icons-9ab8c3f9abc1
[3] https://github.com/vscode-icons/vscode-icons/wiki/Custom/f9dc5ca8b13afa17aa4e849dd64be588978ffe55
[4] https://code.visualstudio.com/api/extension-guides/file-icon-theme
[5] https://nard-studios.vercel.app/blog/customizing-vs-code
[6] https://github.com/vscode-icons/vscode-icons/wiki/FineTuning
