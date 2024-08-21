FROM rust:1.79

EXPOSE 3000

WORKDIR /usr/src/application
COPY . .

RUN rustup update

RUN cargo build -j 1 --release
RUN ln -s ./target/build/git-stats-web .

CMD ["./git-stats-web"]