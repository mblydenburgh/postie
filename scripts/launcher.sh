#!/bin/bash

DIR="$( cd "$( dirname "$0" )" && pwd )"
BIN=gui
DATABASE_URL="$DIR/postie.sqlite"

$DIR/$BIN $DATABASE_URL
