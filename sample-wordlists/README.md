Custom wordlists for typoist.

Drop .txt files in this directory. Each file becomes a word set you can
select with ctrl+w (or by setting word_set = "custom:<filename>" in
settings.toml).

Format:
  - One word or phrase per line
  - Lines starting with # are comments
  - Blank lines are ignored
  - A line can have a trailing # comment: `foo # note`
  - Case and symbols are preserved, so `Box<T>` and `kebab-case` work

Example file: spanish.txt
  # top spanish words
  el
  la
  de
  que
  y
