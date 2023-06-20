#/usr/bin/env bash
set -eux
docker build --tag 'docker_debuild' -f utils/Dockerfile .
DOCKER_ID_RUN=$(docker run -d docker_debuild:latest)
if test -z "$DOCKER_ID_RUN"
then
    echo "\$DOCKER_ID_RUN is empty"
else
    docker cp "${DOCKER_ID_RUN}:/dynotests/target/debian/" bin/
fi
