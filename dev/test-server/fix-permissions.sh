#!/bin/sh
# Runs inside the SFTP container at startup: the shared volume is created
# owned by root (and nginx fills it with its own index.html), so hand the
# upload folder and everything in it to the test user.
chown -R 1001:1001 /home/qed/www
