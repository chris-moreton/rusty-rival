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

# Download Stockfish for Linux
RUN wget -q -O /tmp/stockfish.tar \
    https://github.com/official-stockfish/Stockfish/releases/download/sf_17/stockfish-ubuntu-x86-64-avx2.tar && \
    tar -xf /tmp/stockfish.tar -C /tmp && \
    mv /tmp/stockfish/stockfish-ubuntu-x86-64-avx2 engines/stockfish/stockfish-linux-x86_64 && \
    chmod +x engines/stockfish/stockfish-linux-x86_64 && \
    rm -rf /tmp/stockfish /tmp/stockfish.tar

# Copy scripts and web modules
COPY scripts/compete.py scripts/
COPY web/ web/

# Copy opening book EPD files
COPY engines/epd/*.epd engines/epd/

# Engine version to download (tag name from GitHub releases)
ARG ENGINE_VERSION=latest
ARG ENGINE_NAME=v099-docker

# Download rusty-rival binary from GitHub releases
RUN mkdir -p engines/${ENGINE_NAME} && \
    if [ "$ENGINE_VERSION" = "latest" ]; then \
        wget -q -O engines/${ENGINE_NAME}/rusty-rival \
            https://github.com/chris-moreton/rusty-rival/releases/latest/download/rusty-rival-linux-x86_64; \
    else \
        wget -q -O engines/${ENGINE_NAME}/rusty-rival \
            https://github.com/chris-moreton/rusty-rival/releases/download/${ENGINE_VERSION}/rusty-rival-linux-x86_64; \
    fi && \
    chmod +x engines/${ENGINE_NAME}/rusty-rival

# Store the engine name for runtime
ENV ENGINE_NAME=${ENGINE_NAME}

# Environment variables
ENV PYTHONUNBUFFERED=1

# Default command - can be overridden
ENTRYPOINT ["python3", "scripts/compete.py"]
CMD ["--help"]
