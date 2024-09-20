# Quaver-Bio (QB) #
**A program for updating your [Discord](https://discord.com/) account biography with various information from the [Quaver](https://quavergame.com/) API**

> ## Warning ##
> **Discord ToS explicitly disallows automating a user account in any way.** While I have not personally had any issues, use this at your own risk

### Inspiration ###
I love playing Quaver, and for many months, I had my rank in my Discord bio, however, I had to manually update it, and so I thought, why not automate it?

### Design & Features ###
- Built to be containerized for use with Docker. QB can be configured via environment variables along with a `config.json` file.
- Logs can be sent to [Loki](https://grafana.com/docs/loki/latest/) for log aggregation via [tracing-loki](https://github.com/hrxi/tracing-loki)
(and consequently, Grafana, which can be used to notify you when an error occurs). This is massively overkill, but it was fun to implement.
- Written in Rust for excellent performance, memory usage, and reliability, and because I just like Rust.
- Customizable biography schematic allows you to structure your bio to your preference. 

### Usage ###
When you run QB, you must set some environment variables for it to work:
- `QB_CONFIG_PATH` (Optional) Allows you to specify a path for your config file.
- `QB_LOG_LEVEL` `[TRACE, DEBUG, INFO, WARN, ERROR]` (Optional) Allows you to specify the severity of logs that will be sent to stdout.
- `QB_LOKI_LOG_LEVEL` `[TRACE, DEBUG, INFO, WARN, ERROR]` (Optional) Allows you to specify the severity of logs that will be sent to Loki.
- `QB_LOKI_URL` (Optional) Allows you to specify a Loki endpoint for log aggregation.
- `QB_DISCORD_TOKEN` (Required) Either your discord token or a path to a file that contains it.
When you run the application for the first time, it'll create a config file for you. Here's an example of one:
```json
{
  "quaver_user_id": 1,
  "bio_schema": "Hello, {username}! Your 4K rank is {4k_rank}.",
  "update_interval": 1800
}
```
The configuration values are pretty self-explanatory, but in case you need it:
- `quaver_user_id` The ID of your quaver account (*This can be found by looking at the URL of your profile on the Quaver website)*.
- `bio_schema` The biography schema. This is the text for your discord biography, with optional variables surrounded by `{}` to include certain details of your Quaver account.
- `update_interval` The update interval in seconds (*i.e. how often your discord bio should be updated.
Discord clients cache your biography so there's no reason to make this less than 5 minutes*)

### Supported values ###
These are the values that you can insert into your biography schema by wrapping them in `{}`
- `username` Username
- `country` Country
- `4k_rank` 4k global rank
- `4k_rank_country` 4k country rank
- `4k_total_score` 4k total score
- `4k_ranked_score` 4k ranked score
- `4k_accuracy` 4k accuracy
- `4k_performance_rating` 4k performance rating
- `4k_play_count` Number of songs played in 4k
- `4k_fail_count` Number of songs failed in 4k
- `4k_max_combo` Max combo in 4k
- `7k_rank` 7k global rank
- `7k_rank_country` 7k country rank
- `7k_total_score` 7k total score
- `7k_ranked_score` 7k ranked score
- `7k_accuracy` 7k accuracy
- `7k_performance_rating` 7k performance rating
- `7k_play_count` Number of songs played in 7k
- `7k_fail_count` Number of songs failed in 7k
- `7k_max_combo` Max combo in 7k

> ## Note ##
> When running the application, you may think that your bio hasn't been updated. This is because Discord clients like to cache your bio.
> Restarting your discord client should update it.
> This implies that the updates aren't real-time since your friends won't be able to see the bio change for a while, but there is nothing I can do about it.

> ## Note ##
> Discord has a 190-character limit for biographies. QB will not update your bio if it exceeds this length.
