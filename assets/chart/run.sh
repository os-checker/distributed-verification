#!/usr/bin/bash

set -eoux pipefail

cd assets/chart
sqlite3 -json ../core.sqlite3 <count.sql >count.json
