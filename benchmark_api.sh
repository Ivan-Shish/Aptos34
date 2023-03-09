#!/bin/bash

# TODO: extend this script to benchmark the missing endpoints as
# well as handle other request types, e.g., POST requests.

# Rewrk constants
THREADS=500
CONNECTIONS=500
DURATION=10s

# Command arguments
ARGUMENTS="-t ${THREADS} -c ${CONNECTIONS} -d ${DURATION} --pct"

# API endpoint
ENDPOINT="http://34.28.67.255:8111/v1"

# Benchmark account endpoints
rewrk -h ${ENDPOINT}/accounts/0x1 $ARGUMENTS
rewrk -h ${ENDPOINT}/accounts/0x1/resources $ARGUMENTS
rewrk -h ${ENDPOINT}/accounts/0x1/modules $ARGUMENTS

# Benchmark block endpoints
rewrk -h ${ENDPOINT}/blocks/by_height/10000 $ARGUMENTS
rewrk -h ${ENDPOINT}/blocks/by_version/10000 $ARGUMENTS

# Benchmark event endpoints
rewrk -h ${ENDPOINT}/accounts/0x1/events/3 $ARGUMENTS

# Benchmark general endpoints
rewrk -h ${ENDPOINT}/ $ARGUMENTS

# Benchmark table endpoints (TODO!)

# Benchmark transaction fetching
rewrk -h ${ENDPOINT}/transactions $ARGUMENTS
rewrk -h ${ENDPOINT}/transactions/by_version/10 $ARGUMENTS
rewrk -h ${ENDPOINT}/estimate_gas_price $ARGUMENTS
