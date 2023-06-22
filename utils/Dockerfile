# ==========================================================================================#
FROM rust:slim

RUN apt-get update && apt-get install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev pkg-config libudev-dev libgtk-3-dev  build-essential
RUN cargo install cargo-deb

WORKDIR /dynotests
COPY . .
COPY ./utils/Cargo-release.toml Cargo.toml

RUN cargo build --release
RUN cargo deb
