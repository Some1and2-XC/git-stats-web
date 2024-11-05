FROM rust:1.82

EXPOSE 3000

WORKDIR /usr/src/application
COPY . .

RUN rustup update

RUN cargo build -j 1 --release

CMD ["./target/release/git-stats-web"]
