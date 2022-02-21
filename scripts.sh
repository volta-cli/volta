docker build . -f ./ci/docker/Dockerfile.alpine -t volta-musl-1_0_1-aarch64 --target openssl-1_0_1 --progress=plain --platform=linux/arm64 --no-cache
docker build . -f ./ci/docker/Dockerfile.alpine -t volta-musl-1_1_0-aarch64 --target openssl-1_1_0 --progress=plain --platform=linux/arm64 --no-cache
docker build . -f ./ci/docker/Dockerfile.alpine -t volta-musl-1_0_1-x86_64 --target openssl-1_0_1 --progress=plain --platform=linux/amd64 --no-cache
docker build . -f ./ci/docker/Dockerfile.alpine -t volta-musl-1_1_0-x86_64 --target openssl-1_1_0 --progress=plain --platform=linux/amd64 --no-cache

docker run --volume ${PWD}:/root/workspace --platform linux/arm64 --workdir /root/workspace --rm --init --tty volta-musl-1_0_1-aarch64 /root/workspace/ci/build-with-openssl.sh linux-musl-openssl-1_0_1-aarch64 aarch64-unknown-linux-musl
docker run --volume ${PWD}:/root/workspace --platform linux/arm64 --workdir /root/workspace --rm --init --tty volta-musl-1_1_0-aarch64 /root/workspace/ci/build-with-openssl.sh linux-musl-openssl-1_1_0-aarch64 aarch64-unknown-linux-musl
docker run --volume ${PWD}:/root/workspace --platform linux/amd64 --workdir /root/workspace --rm --init --tty volta-musl-1_0_1-x86_64 /root/workspace/ci/build-with-openssl.sh linux-musl-openssl-1_0_1-x86_64 x86_64-unknown-linux-musl
docker run --volume ${PWD}:/root/workspace --platform linux/amd64 --workdir /root/workspace --rm --init --tty volta-musl-1_1_0-x86_64 /root/workspace/ci/build-with-openssl.sh linux-musl-openssl-1_1_0-x86_64 x86_64-unknown-linux-musl