# Native & Docker Runner Setup

## Full Run

```sh
docker build -f src/docker/python/Dockerfile -t leetcode-python .
RUNNER=docker cargo run --bin runner -- python
RUNNER=native cargo run --bin runner -- python
```

## Configs

```sh
docker run --rm \
    --network=none \
    --memory=256m \
    --cpus=1 \
    -v "/tmp/leetcode/<run-id>:/run:rw" \
    leetcode-rust
```

| Option                             | Purpose                                       |
| ---------------------------------- | --------------------------------------------- |
| `--network=none`                   | Submission cannot access the internet/LAN     |
| `--read-only`                      | Container's root filesystem can't be modified |
| `--cpus=1`                         | Prevent CPU abuse                             |
| `--memory=256m`                    | Prevent memory exhaustion                     |
| `--pids-limit=64`                  | Prevent fork bombs                            |
| `--cap-drop=ALL`                   | Remove Linux capabilities                     |
| `--security-opt=no-new-privileges` | Prevent privilege escalation                  |
| `--user=1000:1000`                 | Don't run submission as root                  |
| `--rm`                             | Container disappears after execution          |
| `-v ...:/run:rw`                   | Only give it its own run directory            |

```sh
docker run --rm \
  --network=none \
  --read-only \
  --cpus=1 \
  --memory=256m \
  --pids-limit=64 \
  --cap-drop=ALL \
  --security-opt=no-new-privileges \
  --user=1000:1000 \
  -v "/tmp/leetcode/<run-id>:/run:rw" \
  leetcode-rust
```

```sh
docker build -f src/Dockerfile -t leetcode-rust .
```

```Dockerfile
RUN useradd --uid 1000 --create-home runner
COPY src/runner.sh /usr/local/bin/runner
COPY src/runner.rust.sh /usr/local/bin/runner.language.sh
RUN chmod +x \
    /usr/local/bin/runner \
    /usr/local/bin/runner.language.sh
USER runner
WORKDIR /run
ENTRYPOINT ["/usr/local/bin/runner"]
```

## Troubleshoot

### Docker/Runner files

```sh
$ tree ./src -P 'Docker*|runner*' --prune
```

### Build

```sh
docker build -f src/docker/rust/Dockerfile -t leetcode-rust .
docker build -f src/docker/python/Dockerfile -t leetcode-python .
docker build -f src/docker/javascript/Dockerfile -t leetcode-node .
```

#### Run

```sh
RUNNER=docker cargo run --bin runner -- rust
RUNNER=docker cargo run --bin runner -- python
RUNNER=docker cargo run --bin runner -- js
```

## Override

```sh
docker run --rm --entrypoint /bin/cat leetcode-python /usr/local/bin/language-runner
```
