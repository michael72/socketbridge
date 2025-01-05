#!/bin/bash

set -e -m

# local ip needs to be 0.0.0.0 inside docker container to expose the tcp to the outside
PORT=${1:-8234}
SOCK=${2:-'/tmp/square_server.sock'}

../../../target/release/socketbridge unix $SOCK 127.0.0.1:$PORT &
SOCK_PID=$!
trap "kill $SOCK_PID || 'socketbridge unix already exited'" SIGINT SIGTERM EXIT
sleep 1

../target/release/square_client $SOCK 1 2 3 78 162 11
