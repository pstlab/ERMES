# --- Stage 1: The Builder ---
FROM rust:1.93-slim-bookworm AS builder

# Install the necessary dependencies
RUN apt-get update && apt-get install -y build-essential libclang-dev git wget unzip curl && rm -rf /var/lib/apt/lists/*

# Install Node.js (version 22.x)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && apt-get install -y nodejs

WORKDIR /usr/src/app

RUN git clone https://github.com/pstlab/ERMES.git .

RUN wget -O clips_642.zip https://sourceforge.net/projects/clipsrules/files/CLIPS/6.4.2/clips_core_source_642.zip/download && \
    unzip clips_642.zip -d clips_temp && \
    mkdir -p clips_source && \
    mv clips_temp/clips_core_source_642/core/* clips_source/ && \
    rm -rf clips_temp clips_642.zip

RUN cargo build --release

WORKDIR /usr/src/app/gui
RUN npm install && npm run build

# --- Stage 2: The Final Image ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/ermes .
COPY --from=builder /usr/src/app/gui/dist ./gui/dist

EXPOSE 3000

CMD ["ermes"]