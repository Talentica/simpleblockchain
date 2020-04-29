FROM ubuntu:18.04 AS simplebc_buildbase

RUN apt-get update \
    && apt-get install -y software-properties-common \
    && add-apt-repository ppa:maarten-fonville/protobuf \
    && apt-get update \
    && apt-get install -y curl git \
    build-essential libssl-dev libsodium-dev libsnappy-dev \
    librocksdb-dev pkg-config clang-7 lldb-7 lld-7 \
    libprotobuf-dev protobuf-compiler \
    python3-pip python3-setuptools
    
ENV PROTOC_ZIP=protoc-3.7.1-linux-x86_64.zip
RUN curl -OL https://github.com/protocolbuffers/protobuf/releases/download/v3.7.1/$PROTOC_ZIP \
    && unzip -o $PROTOC_ZIP -d /usr/local bin/protoc \
    && unzip -o $PROTOC_ZIP -d /usr/local 'include/*' \
    && rm -f $PROTOC_ZIP

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=stable

FROM simplebc_buildbase:latest AS simplebc_build

ENV PATH="${HOME}/.cargo/bin:${PATH}"

COPY . /simpleblockchain/

WORKDIR /simpleblockchain

RUN /root/.cargo/bin/cargo build --release

FROM simplebc_buildbase:latest AS simplebc

COPY --from=simplebc_build /simpleblockchain/target/release /simpleblockchain
