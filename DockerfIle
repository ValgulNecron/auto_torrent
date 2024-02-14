FROM rust:slim-bookworm

RUN cargo install imdl

WORKDIR /auto_torrent

COPY . .

RUN cargo build -r

CMD ["/auto_torrent/target/release/auto_torrent -f /in -o /out -u http://qbittorrent:8080"]