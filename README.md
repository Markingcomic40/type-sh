# TypeSH

Minimal CLI typing test with a fire name.

Started this to learn rust without ai cause man I was so tired of hearing it and missed 'coding'. I feel like it kinda satisfied its purpose at some point so its no longer ai free tho. Ive also come quite to peace w AI ig so I dont even have the energy to write the blog i wanted to write on all that anymore haha. I think as long as you're learning and enjoying building its chill... Have another AI free project now thats a bit more exciting and uh yeah I'll just keep a master branch for when I wanna do old school style on the train or plane or whatever :/

> [typesh.xyz](https://typesh.xyz)

## Install

```sh
cargo install type-sh
```

The installed command is `typesh`:

```sh
typesh
```

Building from source:

```sh
git clone https://github.com/Markingcomic40/type-sh
cd type-sh
cargo run --release
```

## Usage

Figure it out, or ask AI idk bro.

## Word lists

WIP; man i gotta add like a prompt package or sth lmfao.

## Themes

`gruvbox` and `ayu-mirage`, the superior themes (especially the former), ship built in. For your own, go to settings, press enter on the theme row and give it a path to a JSON file like:

```json
{
  "background": [40, 40, 40],
  "text": [235, 219, 178],
  "dim": [146, 131, 116],
  "faint": [80, 73, 69],
  "accent": [250, 189, 47],
  "correct": [235, 219, 178],
  "error": [251, 73, 52],
  "error_extra": [204, 36, 29]
}
```

Every colour is `[r, g, b]`:

| Field         | Used for                                                   |
| ------------- | ---------------------------------------------------------- |
| `text`        | main text                                                  |
| `dim`         | labels, hints and words you haven't typed yet              |
| `faint`       | separators and chart axes (defaults to `dim`)              |
| `accent`      | selections, the timer, your wpm (defaults to `text`)       |
| `correct`     | correctly typed letters (defaults to `text`)               |
| `error`       | wrong letters                                              |
| `error_extra` | letters typed past the end of a word (defaults to `error`) |
| `background`  | optional; leave it out to keep your terminal's own         |

Themes using the old `primary`/`secondary`/`incorrect`/`incorrect_subtle` names still load.

## Configuration

Your last-used settings are saved automatically to:

| Platform | Path                                                |
| -------- | --------------------------------------------------- |
| macOS    | `~/Library/Application Support/type-sh/config.json` |
| Linux    | `~/.config/type-sh/config.json`                     |
| Windows  | `%APPDATA%\type-sh\config.json`                     |

There's no need to edit it directly but fyi ig.

## Why I built this

WIP

## License

MIT — see [LICENSE](LICENSE).
