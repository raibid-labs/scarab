# Hello Notification Plugin

A friendly welcome notification when Scarab starts. Shows time-based greetings and personalized messages.

## Features

- Time-aware greetings (morning, afternoon, evening)
- Random motivational messages
- Personalized with your username
- Configurable via plugin.toml

## Installation

```bash
just plugin-build hello-notification
```

## Configuration

Add to your `~/.config/scarab/config.toml`:

```toml
[[plugins]]
name = "hello-notification"
enabled = true

[plugins.config]
use_time_based = true
use_username = true
```

## Usage

The notification appears automatically when Scarab starts. No user interaction required!

## Development

```bash
just dev-mode hello-notification
```

## Example Output

Morning (7 AM):
```
Good morning, alice!
Ready to get things done?
```

Evening (8 PM):
```
Good evening, bob!
Let's make something awesome!
```

## Customization

Edit the `greetings` list in `hello-notification.fsx` to add your own messages:

```fsharp
let greetings = [
    "Your custom message here!"
    "Another great message!"
]
```

## License

MIT
