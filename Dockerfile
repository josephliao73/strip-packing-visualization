FROM rust:latest AS builder

WORKDIR /app/packing_interface

RUN apt-get update -o Acquire::Retries=3 -o APT::Update::Error-Mode=any && apt-get install -y --no-install-recommends \
    g++ \
    libasound2-dev \
    libegl-dev \
    libfontconfig1-dev \
    libfreetype6-dev \
    libgl-dev \
    libssl-dev \
    libwayland-dev \
    libx11-dev \
    libxcursor-dev \
    libxinerama-dev \
    libxkbcommon-dev \
    libxi-dev \
    libxrandr-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY packing_interface/Cargo.toml packing_interface/Cargo.lock ./
COPY packing_interface/src ./src
COPY packing_interface/requirements.txt ./requirements.txt
RUN cargo build --release

FROM debian:trixie-slim

RUN apt-get update -o Acquire::Retries=3 -o APT::Update::Error-Mode=any && apt-get install -y --no-install-recommends \
    g++ \
    libasound2t64 \
    libegl1 \
    libfontconfig1 \
    libfreetype6 \
    libgl1 \
    libgl1-mesa-dri \
    libegl-mesa0 \
    libvulkan1 \
    mesa-vulkan-drivers \
    libwayland-client0 \
    libx11-6 \
    libxcursor1 \
    libxinerama1 \
    libxkbcommon0 \
    libxi6 \
    libxrandr2 \
    libxkbcommon-x11-0 \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/packing_interface

COPY --from=builder /app/packing_interface/target/release/packing_interface ./packing_interface
COPY packing_interface/src/algorithm_templates ./src/algorithm_templates
COPY packing_interface/src/runner_utils ./src/runner_utils
COPY packing_interface/src/runner_lib ./src/runner_lib
COPY packing_interface/requirements.txt ./requirements.txt

RUN python3 -m pip install --break-system-packages --no-cache-dir -r requirements.txt


ENV LIBGL_ALWAYS_SOFTWARE=1
ENV GALLIUM_DRIVER=llvmpipe
ENV WGPU_BACKEND=vulkan

CMD ["./packing_interface", "python", "cpp"]
