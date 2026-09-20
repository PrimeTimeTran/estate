FROM rust:1.89-bookworm

# Runtime dependencies for all supported languages.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        python3 \
        nodejs \
    && rm -rf /var/lib/apt/lists/*

# Unprivileged execution user.
RUN useradd --uid 1000 --create-home runner

# Main dispatcher.
COPY src/tool/runner.sh /usr/local/bin/runner

# Language-specific runners.
COPY src/tool/runner/rust/runner.sh /usr/local/bin/runner.rust.sh
COPY src/tool/runner/python/runner.sh /usr/local/bin/runner.python.sh
COPY src/tool/runner/javascript/runner.sh /usr/local/bin/runner.javascript.sh

RUN chmod +x \
    /usr/local/bin/runner \
    /usr/local/bin/runner.rust.sh \
    /usr/local/bin/runner.python.sh \
    /usr/local/bin/runner.javascript.sh

USER runner
WORKDIR /run

ENTRYPOINT ["/usr/local/bin/runner"]
