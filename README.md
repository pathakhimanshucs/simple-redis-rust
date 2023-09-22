[![progress-banner](https://backend.codecrafters.io/progress/redis/f67f28a8-e60c-485d-8705-e1766aeadb77)](https://app.codecrafters.io/users/pathakhimanshucs?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own Redis" Challenge](https://codecrafters.io/challenges/redis).

This is a toy Redis clone that's capable of handling
basic commands like `PING`, `SET` (with `px` passive expiry) and `GET`.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

# Running the project

1. Ensure you have `cargo (1.54)` installed locally
1. Run `./spawn_redis_server.sh` to run your Redis server, which is implemented
   in `src/main.rs`. This command compiles your Rust project, so it might be
   slow the first time you run it. Subsequent runs will be fast.
