FROM rust

# install zsh
RUN apt-get update && apt-get upgrade -y && apt-get install -y zsh ripgrep

# install nightly rust
RUN rustup install nightly
RUN rustup default nightly

RUN rustup component add rustfmt
