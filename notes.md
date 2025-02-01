this is just my (py's) random thoughts and notes

## file tree
```
.
├── cauldron/
│   ├── plugins/
│   │   ├── plugin-a/
│   │   │   ├── plugin-a.dll
│   │   │   └── plugin-a-readme.md
│   │   └── plugin-b.dll
│   ├── cauldron.dll
│   ├── cauldron.toml
│   └── loadorder.txt (or equivalent idk yet, maybe toml to be consistent?)
├── version.dll
├── game.exe
└── ...
```

## todos
- replace focus with a nx dxgi hook
  - investigate putting on a toggle for nixxes games
- investigate why closing the game panics
  - probably forgot to free something like a dumbass