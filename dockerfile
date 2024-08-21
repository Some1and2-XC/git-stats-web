FROM rust:1.79

WORKDIR /usr/src/application
COPY . .

RUN rustup update

RUN cargo build --release
RUN ln -s ./target/build/git-stats-web .

CMD ["./git-stats-web"]
