# TypeSH

Minimal CLI typing test. I wanna write a mini blog or sth on why I built this cause it was as a bit of a rest from AI assisted coding entirely (though I'm planning to break that now cause, idk reasons id write about ig. Oh and to be truly factual web is smth i built a whileeeee ago but not as refined so idk I kinda considered it also part of the end of the project and that was built w ai assistance). But yeah idk my feelings on it have changed quite a bit since I wrote my initial thoughts, and I just cant also get into that mood rn yfm but I also want to push this out cause its been like near done for a v0 for ages haha. Been working on some other projects B)

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

`gruvbox` and `ayu-mirage`, the superior themes (especially the former), ship built in. For your own, select `custom` on the theme setting and enter a path to a JSON file like:

```json
{
  "background": [40, 40, 40],
  "primary": [235, 219, 178],
  "secondary": [146, 131, 116],
  "correct": [184, 187, 38],
  "incorrect": [251, 73, 52],
  "incorrect_subtle": [204, 36, 29]
}
```

Every field is `[r, g, b]`. `background` is optional; leave it out to keep your terminal's own bg.

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
