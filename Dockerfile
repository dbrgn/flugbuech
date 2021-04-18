# Build frontend resources in node container
FROM node:12-slim AS frontend-build

# Build with npm
COPY . /build/flugbuech/
RUN cd /build/flugbuech && npm install && npm run build

# Build server resources in rust container
FROM rustlang/rust:nightly-buster

# Install dependencies
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update -q \
 && apt-get -y -q install --no-install-recommends dumb-init \
 && rm -rf /var/lib/apt/lists/*

# Create users and directories
RUN adduser --disabled-password --gecos "" flugbuech
RUN mkdir /flugbuech \
 && chown flugbuech:flugbuech /flugbuech/ \
 && chmod 0700 /flugbuech/
RUN mkdir /static \
 && chown flugbuech:flugbuech /static/ \
 && chmod 0700 /static/

# Copy required files
COPY --chown=flugbuech:flugbuech static /flugbuech/static/
COPY --chown=flugbuech:flugbuech templates /flugbuech/templates/
COPY --chown=flugbuech:flugbuech Rocket.toml /flugbuech/

# Copy generated frontend files
COPY --from=frontend-build --chown=flugbuech:flugbuech /build/flugbuech/static/js/*.component.js /flugbuech/static/js/
COPY --from=frontend-build --chown=flugbuech:flugbuech /build/flugbuech/static/js/maplibre-gl.* /flugbuech/static/js/
COPY --from=frontend-build --chown=flugbuech:flugbuech /build/flugbuech/static/css/maplibre-gl.* /flugbuech/static/css/

# Build binary
COPY . /build/flugbuech/
RUN cd /build/flugbuech \
 && cargo build --release \
 && cp /build/flugbuech/target/release/flugbuech /flugbuech/flugbuech \
 && rm -r /build/flugbuech

# Specify workdir and user
WORKDIR /flugbuech
USER flugbuech

# Create volumes
VOLUME ["/static"]

ENV RUST_BACKTRACE=1
ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["bash", "-c", "rm -rf /static/{js,img,css,svelte}/ && cp -Rv /flugbuech/static/* /static/ && exec ./flugbuech --migrate"]
