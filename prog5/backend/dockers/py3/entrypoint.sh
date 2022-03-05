#!/usr/bin/env bash
timeout --signal=SIGTERM 20s python3 main.py

exit$?