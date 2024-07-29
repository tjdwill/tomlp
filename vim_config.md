# Tj's Vim Setup

Version: 9.1
Dependencies:
    - Node.js
    - The C/C++ toolchain (`make`, `gcc`, etc.)


First, we need a plugin manager. I am currently using `vim-plug` via [the GitHub page](https://github.com/junegunn/vim-plug).
Once this is installed, download the extensions:

## Extensions

0. [Dracula Theme](https://github.com/dracula/vim)
1. [coc.nvim](https://github.com/neoclide/coc.nvim) - 
    - Extension: [coc-rust-analyzer](https://github.com/fannheyward/coc-rust-analyzer)
2. [ALE](https://github.com/dense-analysis/ale)

### CoC Setup

Assume vim-plug and Node.js are both installed and available.

1. specify coc.nvim in your package manager.

```vim
Plug 'neoclide/coc.nvim', {'branch': 'release'},
```

2. Install via `:PlugInstall`
3. Install `coc-rust-analyzer` via `:CocInstall coc-rust-analyzer`
4. Source the [example configuration](https://github.com/neoclide/coc.nvim#example-vim-configuration) in your vimrc or paste it directly. 
    - Important for Tab completion and other features.
5. Add the following to `:CocConfig` to control what is checked by cargo clean.

```json
    "workspace.ignoredFolders": [
      "$HOME",
      "$HOME/.cargo/**",
      "$HOME/.rustup/**"
    ],
```

**For Posterity**: the feature where type hints are displayed as virtual text is called *inlay hinting*. Knowing this saves search time.

### ALE Setup

1. `Plug 'dense-analysis/ale'`
2. Configure via `let g:ale_linters={'rust': ['analyzer']}`
3. Add any additional configuration options as desired. 

### Coc and ALE Interop

These plugins work together, we just need to change the configuration.

**Coc**

1. Open the configuration JSON file `:CocConfig`.
2. Add `"diagnostic.displayByAle": true,` to the file.

**ALE**

1. Add `let g:ale_disable_lsp=1` to the relevant VimScript file (either `.vimrc` or a custom file that you source within `.vimrc`)


