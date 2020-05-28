#### inspiration

coming

---


#### usage

prep...

- git clone project

- cd /path/to/project

- cargo build --release

    (that'll create, in a 'target' directory, the log_rotate binary)

- create an 'env_settings.sh' file like...

        export LOG_ROTATOR__LOG_LEVEL="DEBUG"
        export LOG_ROTATOR__LOGGER_JSON_FILE_PATH="/path/to/logrotate_stuff/log_list.json"
        export LOG_ROTATOR__MAX_ENTRIES="10"

    - the max-entries isn't currently used

    - change the log-level to any of the usual levels

    - sample log_list.json...

            [
              { "path": "/path/to/project_a_logs/project_a.log" },
              { "path": "/path/to/project_b_logs/project_b.log" },
            ]

        (the reason for the "path" dict entries is that I thought I might, at some point in the future, want some logs to have different associated conditions, like 'number-of-backups', 'size-threshold', etc.)


actual call...

- sample crontab entry running every-5-minutes

        */5 * * * * cd /path/to/project; source ../env_settings.sh; ./target/release/log_rotate >> /path/to/log_rotate_rust_logs/log_rotate_rust.log

    (reminder; the meaning of those 6 entries: MIN HOUR DAYOFMONTH MONTH DAYOFWEEK COMMAND)

---


#### notes...

coming
