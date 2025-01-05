# socketbridge

A simple rust application similar to `socat` that bridges a unix socket to a local tcp port and vice versa. 

As opposed to `socat` the `socketbridge` stays running also when the client(s) disconnect.
However it still finishes when the server application that `socketbridge` is connected to as a client finishes.

There are two modes:
- `unix`: creates a UNIX socket and forwards data to a TCP address.
- `tcp`: creates a TCP server and forwards data to a UNIX socket.

Usage:
```sh
unix <UNIX_SOCKET_PATH> <TCP_ADDRESS>
tcp <TCP_PORT> <UNIX_SOCKET_PATH>
```

A sample setup involving an application running inside a docker container that communicates via the bridge to the other application on the host computer is available in the examples directory.
