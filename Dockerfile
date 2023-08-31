FROM rust:1.72

RUN apt-get update && apt-get install -y \
        build-essential \
        pkg-config \
        avr-libc \
        gcc-avr \
        libelf-dev \
        libsimavr-dev \
        libczmq-dev \
        libclang-dev \
        protobuf-compiler

RUN /bin/bash

