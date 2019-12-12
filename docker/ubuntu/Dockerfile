FROM ubuntu:18.04

LABEL maintainer="Michael Lodder <redmike7@gmail.com>"

ENV PATH /root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
ENV LD_LIBRARY_PATH /usr/local/lib

WORKDIR /root

RUN apt-get update 2>&1 > /dev/null \
    && apt-get install -qq -y curl gcc pkg-config libssl1.0.0 libssl-dev libdbus-1-3 libdbus-1-dev 2>&1 > /dev/null \
    && mkdir -p $HOME/.token-website \
    && cd /usr/lib/x86_64-linux-gnu \
    && ln -s libssl.so.1.0.0 libssl.so.10 \
    && ln -s libcrypto.so.1.0.0 libcrypto.so.10 \
    && curl https://sh.rustup.rs -sSf | sh -s -- -y \
    && rustup default nightly