FROM rust:slim-bookworm

RUN apt-get update && \
    apt-get install -y libssl-dev && \
    apt-get install -y libsqlite3-dev && \
    apt-get install -y libpng-dev && \
    apt-get install -y libjpeg-dev && \
    apt-get install -y ca-certificates && \
    # Make sure the package name is correct and available in the repositories
    apt-get install -y qbittorrent && \
    rm -rf /var/lib/apt/lists/*

RUN cargo install imdl

WORKDIR /auto_torrent

COPY . .

RUN cargo build -r

RUN mkdir /in
RUN mkdir /out

CMD ["/auto_torrent/target/release/auto_torrent", "-f", "/in", "-o", "/out", "-u", "http://qbittorrent:8080"]