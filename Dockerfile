# Build stage - compile rusty-rival from source
FROM rust:1.83-slim AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy source code for current version (v000-local)
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

# Build current version
RUN cargo build --release && \
    mkdir -p /engines/v000-local && \
    cp target/release/rusty-rival /engines/v000-local/

# Copy and run the build script for tagged versions
COPY scripts/build_engines.sh /build_engines.sh
RUN chmod +x /build_engines.sh && /build_engines.sh

# Runtime stage
FROM python:3.11-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies
RUN pip install --no-cache-dir \
    python-chess \
    python-dotenv \
    "psycopg[binary]" \
    Flask \
    Flask-SQLAlchemy

# Create directory structure
RUN mkdir -p engines/stockfish engines/epd scripts web results/competitions

# Download Stockfish for Linux (64-bit version for maximum compatibility)
RUN wget -q -O /tmp/stockfish.tar \
    https://github.com/official-stockfish/Stockfish/releases/download/sf_17/stockfish-ubuntu-x86-64.tar && \
    tar -xf /tmp/stockfish.tar -C /tmp && \
    mv /tmp/stockfish/stockfish-ubuntu-x86-64 engines/stockfish/stockfish-linux-x86_64 && \
    chmod +x engines/stockfish/stockfish-linux-x86_64 && \
    rm -rf /tmp/stockfish /tmp/stockfish.tar

# Copy all built engines from builder stage
COPY --from=builder /engines/ engines/
RUN chmod +x engines/*/rusty-rival

# Copy scripts and web modules
COPY scripts/compete.py scripts/
COPY web/ web/

# Copy opening book EPD files
COPY engines/epd/*.epd engines/epd/

# Environment variables
ENV PYTHONUNBUFFERED=1

# Default command
ENTRYPOINT ["python3", "scripts/compete.py"]
CMD ["--help"]
