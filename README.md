# cc-hook

A lightweight wrapper specifically for claude code to trigger notifications or custom commands when tasks complete or permissions are requested.

## What is cc-hook?

`cc-hook` wraps around `claude` executions and monitors its output. It automatically triggers commands or notifications based on regex patterns you define. Perfect for staying informed without constantly watching the terminal!

## Installation

Just run:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/nahco314/cc-hook/releases/latest/download/cc-hook-installer.sh | sh
```

## Usage

Run your usual `claude` command with `cc-hook`:

```bash
cc-hook claude --model opus
```

## Configuration

You can check your default configuration path with:

```bash
cc-hook config-path
```

Then you can edit it and configure cc-hook.

---

By default, the configuration is empty, so you'll need to define your own hooks to trigger notifications or commands.

Here’s a quick example to get started:

```toml
[[hooks]]
name = "permission_prompt"
regex = "Do you want to proceed\\?"
command = "notify-send '[cc-hook] Claude Code requires your permission!'"

[[hooks]]
name = "task_completed"
regex = "^●.*"
command = "notify-send '[cc-hook] Claude Code is saying something.'"
```

Customize your regex patterns and commands as needed.

## Commands

* Show the config file path:

```bash
cc-hook config-path
```

* Use a custom config file:

```bash
cc-hook -c custom-config.toml claude --model opus
```

## Why use cc-hook?

* Never miss important prompts from `claude code`.
* Get notified instantly when tasks finish.
* Lightweight, customizable, and straightforward.

## Contributing

Contributions, issues, and feature requests are all welcome!

In fact, this tool is 100% made by claude code.
So the code style and comments may be a little strange. So even small refactorings and comment fixes are welcome :)

## License

MIT
