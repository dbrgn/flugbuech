###### FRONTEND ######

# Build frontend resources in node container
FROM node:23-slim AS frontend-build

# Build with npm
COPY frontend /build/flugbuech/
RUN cd /build/flugbuech && npm install && npm run build


###### BACKEND ######

# Build server resources in rust container
FROM docker.io/rust:1-buster AS backend-build

# Build binary
COPY . /build/flugbuech/
RUN cd /build/flugbuech && cargo build --release


###### RUNTIME ######

FROM debian:11-slim

# Install init process and other dependencies
RUN apt-get update -q \
 && apt-get -y -q install --no-install-recommends dumb-init libpq5 \
 && rm -rf /var/lib/apt/lists/*

# Create users and directories
RUN addgroup --gid 1000 flugbuech \
 && adduser --disabled-password --gecos "" --uid 1000 --gid 1000 flugbuech
RUN mkdir /flugbuech \
 && chown flugbuech:flugbuech /flugbuech/ \
 && chmod 0700 /flugbuech/
RUN mkdir /static \
 && chown flugbuech:flugbuech /static/ \
 && chmod 0700 /static/

# Copy backend files
COPY --from=backend-build --chown=flugbuech:flugbuech /build/flugbuech/target/release/flugbuech-api /flugbuech/flugbuech-api
COPY --from=backend-build --chown=flugbuech:flugbuech /build/flugbuech/Rocket.toml /flugbuech/Rocket.toml

# Copy frontend files
COPY --from=frontend-build --chown=flugbuech:flugbuech /build/flugbuech/build /flugbuech/static

# Specify workdir and user
WORKDIR /flugbuech
USER flugbuech

# Create volumes
VOLUME ["/static"]

ENV RUST_BACKTRACE=1
ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["/bin/bash", "-c", "rm -rf /static/* && cp -Rv /flugbuech/static/* /static/ && exec ./flugbuech-api --migrate"]
