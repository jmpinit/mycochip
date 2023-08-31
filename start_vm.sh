#!/bin/bash

ZMQ_PORT=6723
WEB_PORT=8000
docker run -it --rm \
    -v "$(pwd)":/root/mycochip \
    -p $ZMQ_PORT:$ZMQ_PORT \
    -p $WEB_PORT:$WEB_PORT \
    mycochip

