FROM rust:slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates pkg-qBittorrent.conf \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install imdl

WORKDIR /auto_torrent

COPY . .

RUN cargo build -r

RUN mkdir /in
RUN mkdir /out

CMD ["/auto_torrent/target/release/auto_torrent", "-f", "/in", "-o", "/out", "-u", "http://qbittorrent:8080"]