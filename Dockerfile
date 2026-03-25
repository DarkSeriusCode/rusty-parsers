FROM rust:alpine
WORKDIR /usr/src/app

COPY . .

RUN cargo b --release -j12

CMD ["cargo", "r", "--release"]
