FROM docker.io/node:24-trixie-slim


# Install development dependencies
RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get -y install --no-install-recommends \
    build-essential pkg-config musl-tools ca-certificates \
    neovim less git curl wget unzip ripgrep locales file procps net-tools \
    libpq-dev \
 && apt-get clean && rm -rf /var/lib/apt/lists/*

# Locale setup
RUN echo 'en_US.UTF-8 UTF-8' > /etc/locale.gen \
 && locale-gen en_US.UTF-8 \
 && update-locale LANG=en_US.UTF-8
ENV LANG=en_US.UTF-8
ENV LC_ALL=en_US.UTF-8

# Create directories with user ownership
# Note: VS Code Dev Containers will automatically adjust UID/GID via updateRemoteUserUID
RUN mkdir -p /code && chown node:node /code
RUN mkdir -p /opt/rust && chown node:node /opt/rust
RUN mkdir -p /home/node/pgdata && chown node:node /home/node/pgdata

# Switch to non-root user
USER node

# Install Rust through Rustup
ENV RUSTUP_INIT=/opt/rust/rustup-init.sh \
    RUSTUP_HOME=/opt/rust/rustup \
    CARGO_HOME=/opt/rust/cargo
COPY rustup-init.sh $RUSTUP_INIT
RUN $RUSTUP_INIT -y --default-toolchain stable --profile default
ENV PATH="$CARGO_HOME/bin:$PATH"

# Install Rust tools
RUN cargo install -f diesel_cli --no-default-features --features postgres

# Create persistent config directory with default ownership
RUN mkdir -p ~/.config-persistent/

# Create OpenCode directories with default ownership
RUN mkdir -p ~/.config/opencode/ \
 && mkdir -p ~/.local/share/opencode/

ENTRYPOINT ["/bin/bash"]
