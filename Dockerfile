FROM rust:1.61

RUN apt update -y
RUN apt install -y libxcb-shape0-dev libxcb-xfixes0-dev

WORKDIR /usr/src/tomb
COPY . .

RUN cargo install --path .

ENTRYPOINT ["bash"]