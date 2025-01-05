#!/bin/bash

set -e -m

export PORT=${1:-8234}

cd ../../..
# build a local socketbridge
cargo build --release
cd -

# run the socketbridge and the square-server inside the docker container
cd ..
# build a local square_client
cargo build --release
export DOCKER_BUILDKIT=1
docker build -t socketbridge --progress=plain ../..
docker build -t square-server --progress=plain .

docker run -p $PORT:$PORT square-server $PORT &

cd -

# run square-client and the socket bridget on the local computer
sleep 5
./start_client.sh $PORT &
CLIENT_PID=$!
trap "kill $CLIENT_PID || echo 'client already exited'" SIGINT SIGTERM EXIT

# bring docker run to foreground
fg %1

