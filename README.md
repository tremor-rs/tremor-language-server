# Tremor Language Server (Trill)

WIP server for use with editors and IDEs, when working with tremor's languages. Follows [language server protocol](https://microsoft.github.io/language-server-protocol/).


## Features

#### Diagnostics

tremor-script interpreter errors (as you type or on file save), with hints for fixing (as applicable)

nice-to-have: apply fix suggestions from errors

#### Completion

code completion (as you type/on-demand) for module functions -- function names with signature/doc info as well as placeholders for arguments.

nice-to-have: code completion for variables as well as other language constructs

#### Hover

diagnostics and function documentation on mouse hover (or editor command)

nice-to-have: documentation for variables (eg: assignment info)

#### Navigation

nice-to-have: find all references, symbol search

for later: Go to definiton, peek definition, symbol list (when tremor script has functions)

#### Refactoring

nice-to-have: rename all references


## Quickstart

```sh
cd ~/repos # should be the same folder that holds the main tremor-runtime repo (version >= 0.6)
git clone git@github.com:wayfair-incubator/tremor-language-server.git
cd tremor-language-server

# during development
cargo build
ln -s target/debug/tremor-language-server ~/bin/ # or anywhere in your $PATH

# or install the release build
cargo install --path . --root ~/ # make sure ~/bin/ is in your $PATH
```

### VS Code setup

Follow instructions at:

https://github.com/wayfair-incubator/tremor-vscode

### Vim setup

Prerequisite: install https://github.com/wayfair-incubator/tremor-vim so that vim is aware of tremor filetypes (you also get syntax highlighting as a bonus).

For use with vim, we have a forked version of [ale](https://github.com/dense-analysis/ale) that can interact with the tremor language server:

https://github.com/wayfair-incubator/ale/tree/tremor

Follow the plugin installation instructions. If you are using [vim-plug](https://github.com/junegunn/vim-plug), this will do:

```vim
Plug 'wayfair-incubator/ale', { 'branch': 'tremor' }
```

Vim and ale settings that work nice with the tremor language server:

```vim
" completion menu
set completeopt=menuone,longest,popup " always show the menu, insert longest match, use popup window for extra info
"set completepopup=border:off          " remove the border from the completion popup window

" turn on omnicomplete based on ale
set omnifunc=ale#completion#OmniFunc

" enable ale completion (as you type), where available
"let g:ale_completion_enabled = 1

" turn on vim mouse support in all modes (for hover info)
set mouse=a

" show hover information on mouse over (vim mouse support should be turned on)
" xterm2 makes hover work with tmux as well
let g:ale_set_balloons = 1
set ttymouse=xterm2

" only run linters named in ale_linters settings
let g:ale_linters_explicit = 1

" active linters
let g:ale_linters = {
\   'tremor': ['tremor-language-server'],
\   'trickle': ['tremor-language-server'],
\}

" when to run linting/fixing. choose as desired
"
" aggressive
let g:ale_fix_on_save = 1
let g:ale_lint_on_text_changed = 'always'
let g:ale_lint_on_enter = 1
let g:ale_lint_on_insert_leave = 1
"
" conservative
"let g:ale_lint_on_text_changed = 'never'
"let g:ale_lint_on_enter = 0
"let g:ale_lint_on_insert_leave = 0

" key mappings
nmap <silent> <leader>j <Plug>(ale_next_wrap)
nmap <silent> <leader>k <Plug>(ale_previous_wrap)
nmap <silent> <leader>/ <Plug>(ale_hover)
nmap <silent> <leader>? <Plug>(ale_detail)
nmap <silent> <leader>] <Plug>(ale_go_to_definition)
nmap <silent> <leader># <Plug>(ale_find_references)
```

You might want to show ALE counters in your vim status line. If you are using vim [lightline](https://github.com/itchyny/lightline.vim):

```vim
" for showing linter errrors/warnings. depends on lightline-ale
let g:lightline.component_expand = {
    \  'linter_checking': 'lightline#ale#checking',
    \  'linter_warnings': 'lightline#ale#warnings',
    \  'linter_errors': 'lightline#ale#errors',
    \  'linter_ok': 'lightline#ale#ok',
    \ }
let g:lightline.component_type = {
    \  'linter_checking': 'left',
    \  'linter_warnings': 'warning',
    \  'linter_errors': 'error',
    \  'linter_ok': 'left',
    \ }
let g:lightline#ale#indicator_checking = ''
let g:lightline#ale#indicator_warnings = '▲'
let g:lightline#ale#indicator_errors = '✗'
let g:lightline#ale#indicator_ok = '✓'

" configure lightline components
let g:lightline.active = {
    \   'left':  [ ['mode', 'paste'],
    \              ['fugitive', 'readonly', 'filename', 'modified'] ],
    \   'right': [ [ 'lineinfo' ],
    \              [ 'percent' ],
    \              [ 'fileformat', 'fileencoding', 'filetype' ],
    \              ['linter_checking', 'linter_errors', 'linter_warnings', 'linter_ok' ] ]
    \ }

" ale indicators (aligned with indicators used in lightline-ale)
" 2 chars to cover the full sign width
let g:ale_sign_warning = '▲▲'
let g:ale_sign_error = '✗✗'
```

For more ale setup and vim configuration:

https://github.com/anupdhml/dotfiles/blob/virtualbox_new/data/.vimrc

If you prefer not to use ale, these vim plugins should also work well as the server client:

* https://github.com/prabirshrestha/vim-lsp
* https://github.com/autozimu/LanguageClient-neovim

#### Notes

* If you are using vim from terminal and not seeing error diagnostics or function docs on hover,
  check if the vim version you are using has been compiled with balloon support -- output of
  `vim --version` should show items `+balloon_eval` and `+balloon_eval_term`. If not, you will
  need to find a vim package for your environment with the support baked in (or compile vim yourself).
  Eg: for mac, this may mean installing `macvim` via homebrew.
* By default, vim's omni-completion items (eg: tremor function names after typing `module_name::`) are
  triggered via `Ctrl-x Ctrl-o`, while normal keyword completion is via `Ctrl-p/Ctrl-n`. If you prefer a
  more accessible keybinding for these (eg: `Tab`), have a look at extensions like
  [VimCompletesMe](https://github.com/ajh17/VimCompletesMe), or [Supertab](https://github.com/ervandew/supertab).

## TODO

* integration for emacs
* support parallel edits for trickle and tremor files
* improve debugging
* add tests
* ability to handle multiple script errors
* use simd-json in tower and json rpc crates?
* distribution without compiling
