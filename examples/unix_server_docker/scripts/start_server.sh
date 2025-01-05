#!/bin/bash

set -e -m

# local ip needs to be 0.0.0.0 inside the docker container to expose interface to the outside
PORT=$1

./square_server &
sleep 2
echo "started square server"

echo "running bridge on port $port" 
./socketbridge tcp 0.0.0.0:$PORT /tmp/square_server.sock &
SOCK_PID=$!
trap "kill $SOCK_PID || 'socketbridge tcp already exited'" SIGINT SIGTERM EXIT

echo "switching to foreground"
fg %1

echo "square server exited"



