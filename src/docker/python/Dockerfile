FROM python:3.13-slim

RUN useradd --uid 1000 --create-home runner

COPY src/runner.sh /usr/local/bin/runner
COPY src/runner/python/runner.sh /usr/local/bin/language-runner

RUN chmod +x \
    /usr/local/bin/runner \
    /usr/local/bin/language-runner

USER runner

WORKDIR /run

ENTRYPOINT ["/usr/local/bin/runner"]
