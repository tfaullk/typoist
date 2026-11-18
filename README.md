# typoist
A terminal typing practice application.  
Built in Rust with [Ratatui](https://github.com/ratatui/ratatui).

## LEADERBOARDS
| Rank | Username                                                                              | PB (WPM) | Ed25519 Identity                                                 |
| ---- | ------------------------------------------------------------------------------------- | -------- | ---------------------------------------------------------------- |
| 1    | tfaullk             | 106.0    | 6b49e9d13892671bfdbfd05a299151f4a2cbbb753d2437d46f26f10cedb14c8c |
| 2    | YesYes              | 103.1    | 7099ddf0be967f196dcdbd02ba22f3a4acbdf21d463dfda0200871fdcbf2e3fb |




## Features

- **Variable time modes**: time 15/30/60/120s
- **Four word sets**: plain english, rust/code keywords, english with punctuation and capitals, code with symbols
- **20 built-in themes**: dracula, nord, gruvbox, catppuccin, tokyonight and more, all live-previewable
- **Custom themes**: drop a `.toml` file (see themes section for formatting) in the themes folder or import one from anywhere
- **Custom Wordlists**: drop a `.txt` file in the wordlists folder or import one from anywhere
- **Live stats**: wpm, accuracy, and a consistency score based on your per-second wpm samples
- **Persisted settings**: your theme, mode, and word set are saved between runs
- **Simple updates**: typoist --update and typoist --check-update are useful tools built-in to keep you up to date
- **Custom Keybinds**: customize your keybinds
- **Leaderboards with verification**: Submit personal bests (scroll down for more) and have them verified to ensure fair play
- **Stenography Mode**: Practice stenography-style typing with support for the leaderboard system

## Install (Windows)

Use the install.ps1 script.
Open a powershell window and type
``` cd Downloads ``` 

then,

``` powershell -ExecutionPolicy Bypass -File .\Install.ps1 ```

Then, run ``` typoist ``` from a cmd or powershell.

## Install (Linux)

Make sure you have a recent Rust toolchain (rustc 1.101.0 or higher), and have cargo in your PATH then:

```sh
mkdir -p ~/.local/bin
curl -fL https://github.com/tfaullk/typoist/releases/download/2.0.0/typoist-2.0.0-x86_64-unknown-linux-musl -o ~/.local/bin/typoist
chmod +x ~/.local/bin/typoist
```

Add `~/.local/bin` to PATH if it isnt there.
**Bash / Zsh:**

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc   # or ~/.zshrc
source ~/.bashrc
```

```zsh
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

```fish
fish_add_path ~/.local/bin
```

Then just run `typoist` from anywhere.

## Usage

```sh
typoist
```

You can also jump straight into a specific setup:

```sh
typoist --theme nord --words code --mode "time 60"
```

### Commands


| Command                 | What it does                                                       |
| ----------------------- | ------------------------------------------------------------------ |
| `-t, --theme <NAME>`    | Start with a specific theme                                        |
| `-w, --words <SET>`     | Word set: `english`, `code`, `english+punctuation`, `code+symbols` |
| `-m, --mode <MODE>`     | Mode: `time 15|30|60|120`                                          |
| `--list-themes`         | List installed themes and exit                                     |
| `--list-word-sets`      | List available word sets and exit                                  |
| `--import-theme <FILE>` | Import a theme file into your themes directory                     |
| `--no-save`             | Don't persist settings for this session                            |
| `-h, --help`            | Show help                                                          |
| `-v, --version`         | Show current version                                               |
| `--update`              | Update to latest version                                           |
| `--check-update`        | Check for updates                                                  |
| `--verify-pb`           | Verifies personal best from hash (see more below)                  |
| `--identity`            | Shows cryptographic identity for verification                      |
| `--export-pb`           | Exports PB (e.g typoist --export-pb "steno 30" > steno_30.jsonl)   |
| `--pubkey`              | Companion flag for --verify-pb, allows you to supply the ED25519   |
|                         | key to verify.                                                     |


## Keybinds

### During a test


| Key             | Action                                        |
| --------------- | --------------------------------------------- |
| `tab`           | New test with fresh words                     |
| `esc`           | Restart with the same words — or quit if idle |
| `backspace`     | Delete your last keystroke                    |
| `ctrl+t`        | Open the theme picker                         |
| `ctrl+w`        | Cycle word set                                |
| `ctrl+n`        | Cycle mode                                    |
| `ctrl+l`        | Toggle live WPM                               |
| `ctrl+a`        | Toggle accuracy display                       |
| `ctrl+h` / `F1` | Help screen                                   |


### Theme picker


| Key                              | Action                        |
| -------------------------------- | ----------------------------- |
| `↑` / `↓` or `ctrl+p` / `ctrl+n` | Preview themes (wraps around) |
| `enter`                          | Confirm and save the theme    |
| `esc`                            | Cancel and go back            |


### Results screen


| Key             | Action           |
| --------------- | ---------------- |
| `tab` / `enter` | Start a new test |
| `esc`           | Quit             |


Anywhere: `ctrl+q` or `ctrl+c` quits.

## Themes

Themes live in your config directory (`~/.config/typoist/themes` on Linux, the equivalent elsewhere) as simple TOML files:

```toml
name = "my-theme"
background = [40, 42, 54]
foreground = [248, 248, 242]
correct = [80, 250, 123]
incorrect = [255, 85, 85]
pending = [98, 114, 164]
cursor = [189, 147, 249]
accent = [139, 233, 253]
sub = [98, 114, 164]
```

You can copy an existing theme and tweak the RGB values, or import one directly:

```sh
typoist --import-theme ~/Downloads/cool-theme.toml
```

## Leaderboards

If you want to be put on a leaderboard that will be in this very README.md, submit your pbs.jsonl file to me. (coming soon)
On Windows this can be found at `%APPDATA%\Roaming\typoist\pbs.jsonl`
And on Linux this can be found at `~/.config/typoist/pbs.jsonl`

## License

Licensed under the **PolyForm Noncommercial License 1.0.0**
