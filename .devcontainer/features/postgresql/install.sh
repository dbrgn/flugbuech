#!/usr/bin/env bash
set -eu

# Feature options
PG_MAJOR_VERSION="${VERSION:-latest}"

# Other variables
INSTALL_PREFIX="/usr/local/pgsql"
TARGET_USER=$_REMOTE_USER

echo "Installing PostgreSQL for user $TARGET_USER..."

# Install build dependencies
apt-get update \
   && DEBIAN_FRONTEND=noninteractive apt-get -y install --no-install-recommends \
     build-essential pkg-config ca-certificates curl \
     libicu-dev libreadline-dev zlib1g-dev libpq5 libpq-dev libssl-dev \
   && apt-get clean && rm -rf /var/lib/apt/lists/*

# Determine latest version
echo "Fetching latest version..."
PG_VERSION=$(curl --proto '=https' --tlsv1.3 -LSsf "https://www.postgresql.org/ftp/source/" | grep -oP "v${PG_MAJOR_VERSION}\.\d+" | sort -V | tail -n1)
PG_VERSION="${PG_VERSION#v}"
if [ -z "$PG_VERSION" ]; then
    echo "Failed to determine latest PostgreSQL version"
    exit 1
fi
echo "Installing PostgreSQL version: $PG_VERSION"

# Create build dir
mkdir /tmp/pg && pushd /tmp/pg

# Download and unpack sourcecode
TARBALL_NAME="postgresql-${PG_VERSION}.tar.gz"
DOWNLOAD_URL="https://ftp.postgresql.org/pub/source/v${PG_VERSION}/postgresql-${PG_VERSION}.tar.gz"
echo "Downloading from: $DOWNLOAD_URL"
curl --proto '=https' --tlsv1.3 -LSsf "$DOWNLOAD_URL" -o $TARBALL_NAME
tar xzf $TARBALL_NAME
rm $TARBALL_NAME
cd postgresql-${PG_VERSION}

# Configure with a user-local prefix
./configure --prefix="$INSTALL_PREFIX" --with-openssl
make -j$(nproc)
make install

# Clean up build files
popd
rm -rf /tmp/pg

# Add to PATH for all users
echo "export PATH=$INSTALL_PREFIX/bin:\$PATH" >> /etc/profile.d/postgresql.sh
source /etc/profile.d/postgresql.sh

# Add start script
STARTSCRIPT=/usr/local/bin/pg-start.sh
cat > $STARTSCRIPT << 'EOF'
#!/usr/bin/env bash
export PATH=/usr/local/pgsql/bin:$PATH
PGDATA="${PGDATA:-$HOME/pgdata}"

if [ ! -d "$PGDATA" ] || [ ! -f "$PGDATA/PG_VERSION" ]; then
    echo "Initializing PostgreSQL data directory at $PGDATA..."
    initdb -D "$PGDATA"
fi

pg_ctl -D "$PGDATA" -l "$PGDATA/logfile" start
EOF
chmod +x $STARTSCRIPT

# Verify installation
if command -v pg_ctl &> /dev/null; then
    echo "PostgreSQL installed successfully!"
    pg_ctl --version
else
    echo "PostgreSQL installation failed"
    exit 1
fi
