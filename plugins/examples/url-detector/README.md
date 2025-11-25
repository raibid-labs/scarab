# URL Detector Plugin

Automatically detects URLs in terminal output and notifies you. Great for catching links in build logs, test output, or general terminal usage.

## Features

- Detects HTTP and HTTPS URLs
- Filters out localhost URLs
- Shows notifications for detected URLs
- Configurable detection thresholds

## Installation

```bash
just plugin-build url-detector
```

## Configuration

Add to your `~/.config/scarab/config.toml`:

```toml
[[plugins]]
name = "url-detector"
enabled = true

[plugins.config]
notify_on_detection = true
max_urls_in_notification = 3
min_url_length = 10
```

## Usage

The plugin runs automatically. Try these commands to test:

```bash
echo "Check out https://github.com/raibid-labs/scarab"
curl -I https://example.com
git clone https://github.com/user/repo.git
```

## Development

```bash
just dev-mode url-detector
```

## How It Works

1. Monitors all terminal output in real-time
2. Uses regex to detect URL patterns
3. Filters out common false positives (localhost, etc.)
4. Shows a notification with detected URLs
5. Logs all detections for review

## Performance

- Uses compiled regex for speed
- Processes 100k+ lines/sec
- Minimal impact on terminal performance

## Future Enhancements

- Click to open URLs in browser
- URL history tracking
- Domain-based filtering
- Automatic URL shortening

## License

MIT
