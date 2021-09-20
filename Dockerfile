###### FRONTEND ######

# Build frontend resources in node container
FROM node:16-slim AS frontend-build

# Build with npm
COPY . /build/flugbuech/
RUN cd /build/flugbuech && npm install && npm run build


###### BACKEND ######

# Build server resources in rust container
FROM docker.io/rust:1-buster AS backend-build

# Build binary
COPY . /build/flugbuech/
RUN cd /build/flugbuech \
 && cargo build --release \
 && mkdir /out \
 && cp /build/flugbuech/target/release/flugbuech /out/flugbuech \
 && cp -R /build/flugbuech/static /out/static \
 && cp -R /build/flugbuech/templates /out/templates \
 && cp -R /build/flugbuech/Rocket.toml /out/Rocket.toml


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
COPY --from=backend-build --chown=flugbuech:flugbuech /out/flugbuech /flugbuech/flugbuech
COPY --from=backend-build --chown=flugbuech:flugbuech /out/static /flugbuech/static/
COPY --from=backend-build --chown=flugbuech:flugbuech /out/templates /flugbuech/templates/
COPY --from=backend-build --chown=flugbuech:flugbuech /out/Rocket.toml /flugbuech/

# Copy frontend files
COPY --from=frontend-build --chown=flugbuech:flugbuech /build/flugbuech/static/js/*.component.js /flugbuech/static/js/
COPY --from=frontend-build --chown=flugbuech:flugbuech /build/flugbuech/static/js/maplibre-gl.* /flugbuech/static/js/
COPY --from=frontend-build --chown=flugbuech:flugbuech /build/flugbuech/static/css/maplibre-gl.* /flugbuech/static/css/

# Specify workdir and user
WORKDIR /flugbuech
USER flugbuech

# Create volumes
VOLUME ["/static"]

ENV RUST_BACKTRACE=1
ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["/bin/bash", "-c", "rm -rf /static/{js,img,css,svelte}/ && cp -Rv /flugbuech/static/* /static/ && exec ./flugbuech --migrate"]
