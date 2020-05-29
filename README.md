### inspiration...

I'm a software developer, and use our redhat servers' `logrotate` to auto-rotate the log-files of my apps.

For local development, I've used the Mac's version of logrotate, which changed a few years ago to `newsyslog`, which I used for some years. At some point, maybe 2 or so years ago, likely due to some OS upgrade --- that stopped working for me. And because I tend to upgrade soon in the cycle, all I could find online were a few similar reports, but no solutions.

I briefly considered writing my own limited solution in python, my main programming language these days. I'm sure there are `newsyslog` solutions now, but while going through [The Book](https://doc.rust-lang.org/stable/book/), learning a bit of [Rust](https://www.rust-lang.org/), I was casting around for a small useful project, and thought of that logrotate idea --- thus this this app.

---


### usage...

prep...

- `mkdir ./log_rotate_stuff`

- `cd ./log_rotate_stuff`

- `git clone https://github.com/birkin/log_rotate.git ./log_rotate`

- `cd ./log_rotate`

- `cargo build --release`

    (so, obviously, this assumes you've installed rust.)

    (that'll create, in a 'target' directory, the `log_rotate` binary.)

- create an `env_settings.sh` file in the `log_rotate_stuff` directory like...

        export LOG_ROTATOR__LOG_LEVEL="DEBUG"
        export LOG_ROTATOR__LOGGER_JSON_FILE_PATH="/path/to/logrotate_stuff/log_list.json"
        export LOG_ROTATOR__MAX_ENTRIES="10"

    - the max-entries isn't currently used

    - change the log-level to any of the usual levels

    - sample `log_list.json`...

            [
              { "path": "/path/to/project_a_logs/project_a.log" },
              { "path": "/path/to/project_b_logs/project_b.log" }
            ]

        (watch that there is no final comma, or the json won't be valid.)

        (the reason for the "path" dict entries is that I thought I might, at some point in the future, want some logs to have different associated conditions, like 'number-of-backups', 'size-threshold', etc.)


actual call...

- sample crontab entry running every-5-minutes

        */5 * * * * cd /path/to/project; source ../env_settings.sh; ./target/release/log_rotate >> /path/to/log_rotate_rust_logs/log_rotate_rust.log

    (reminder; the meaning of those 6 entries: MIN HOUR DAYOFMONTH MONTH DAYOFWEEK COMMAND)

---


### notes...

- this assumes a log-rotation suffix-pattern like

        foo.log
        foo.log.0
        foo.log.1
        etc.
        foo.log.9

- currently the MAX_ENTRIES setting isn't actually used; it's hardcoded to `x.log.9` and then that gets removed.

- currently the max-size threshold is hardcoded to 250K.

- the function that copies the existing log file to its next-incremented suffix incorporates the root log filename. Why? Because a few of my log directories contain two different sets of logs. Example: the `project_a_logs` directory will contain, say, `project_a_webapp.log` entries as well as `project_a_indexer.log` entries --- so targeting the `/path/to/project_a_logs/project_a_webapp.log` will parse out the root filename `project_a_webapp` to only process those logs, while leaving the various `project_a_indexer.log` entries alone (until they're ready to be processed in a completely separate step)

- even lightly experienced rustaceans may cringe at me turning everything into Strings to return to calling fuctions. This was to avoid the ownership/borrowing/lifetime errors while grokking those concepts (I'm only halfway through The Book).

- similarly cringeworthy is my explicit usage of unwrap_or_else() all over the place. Again, it's part of my concept-absorption process.

- re `env_logger`...

    - it drives me nuts that the log-entries created by the `env_logger` logger have UTC timestamps; I assume I'm overlooking the way to get them to output to localtime.

    - unlike most of the tutorial entries, I switched the output to stdout so that cron could output the operation of this app to --- drum-roll --- log files (yes, I have this app rotating them).

    - it seems like there should be a way to specify the envar used by `env_logger` to set the log-level, but all I could find was that it had to be set to `RUST_LOG`. Since I wanted to use the `LOG_ROTATOR__` prefix for this project's envars, I implemented the hack of setting a `RUST_LOG` envar from the `LOG_ROTATOR__LOG_LEVEL` envar.

- This was _fun_. It took a _looong_ time to do the _simplest_ of things, because I was, and am, absorbing unfamiliar concepts. I learned a good smattering of [Go](https://golang.org) a few years ago, and made more progress more quickly. But I can see why folk love Rust. The compiler is a wonder. The usage of Result/Option forces situations to be handled or at least acknowledged up-front, which, even with experience, I skip over in python. And the speed is breathtaking. This is only the beginning.

---

