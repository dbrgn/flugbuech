FROM rustlang/rust:nightly-buster

# Create users and directories
RUN mkdir /flugbuech \
 && adduser --disabled-password --gecos "" flugbuech \
 && chown flugbuech:flugbuech /flugbuech/ \
 && chmod 0700 /flugbuech/

# Create volumes
VOLUME ["/flugbuech/static"]

# Copy required files
COPY --chown=flugbuech:flugbuech static /flugbuech/static/
COPY --chown=flugbuech:flugbuech templates /flugbuech/templates/
COPY --chown=flugbuech:flugbuech Rocket.toml /flugbuech/

# Build binary
COPY . /tmp/flugbuech/
RUN cd /tmp/flugbuech \
 && cargo build --release \
 && cp /tmp/flugbuech/target/release/flugbuech /flugbuech/flugbuech \
 && rm -r /tmp/flugbuech

# Specify workdir and user
WORKDIR /flugbuech
USER flugbuech

CMD [ "./flugbuech", "--migrate" ]
