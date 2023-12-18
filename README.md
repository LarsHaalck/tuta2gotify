# tuta2gotify

A small tool to relay tuta messages to a gotify instance.
The tool doesn't persist a session and relays only unread mails from tuta to gotify and marks them as read afterwards.

## Usage
```
tuta2gotify

USAGE:
    tuta2gotify

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config-file>

SUBCOMMANDS:
    help      Prints this message or the help of the given subcommand(s)
    verify    Wait for incoming device verifications
```

```bash
cargo run --release
```
where config.toml is the config file. See `config.sample.toml`.
Default values are commented out.

## Configuration

### Account
| Variable      | Meaning                                              | Default Value |
| ------------- | -------------                                        | ------------- |
| email_address | email address of your tuta account                   | N/A           |
| password      | password of the account                              | N/A           |
| watch_spam    | if the spam folder should also be monitored          | `false`       |
| show_name     | if the display name of the email should be decrypted | `false`       |
| show_subject  | if the subject of the email should be decrpyted      | `false`       |
| show_body     | if the body of the email should be decrypted         | `false`       |

> [!CAUTION]
> Note that you possibly decrypt sensitive information and relay them to a gotify instance!

### Gotify
| Variable      | Meaning                        | Default Value                                                    |
| ------------- | -------------                  | -------------                                                    |
| url           | url of the gotify server       | N/A                                                              |
| token         | app token for the bot          | N/A                                                              |
| format        | format string of the html part | `""New Mail from {{name}} <{{address}}>: {{subject}}\n{{body}}"`

Available template tokens are `name, address, subject, body`.

Instead of a supplied config, all values can also be set using environtmen variables.
Tuta account variables are prefixed with `T2G_ACCOUNT_`, e.g. `T2G_ACCOUNT_EMAIL_ADDRESS`, while gotify variable are prefixed with `T2G_GOTIFY_`.

## Docker
Modify `.t2g.sample.env`, save it as `.t2g.env` and run `docker compose up -d` to run the server.
