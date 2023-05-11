#!/usr/bin/env python3

# Copyright © Aptos Foundation
# SPDX-License-Identifier: Apache-2.0

# Run a fully isolated indexer GRPC setup against a local single node testnet
# requires pyyaml

import os
import signal
import subprocess
import yaml
import requests
import time

TESTNET_DATA_DIR = "./test_indexer_grpc_testnet"
LOCAL_TESTNET_REST_API_ADDR = "http://localhost:8080"
PATH = "./target/debug/"

def run_testnet() -> subprocess.Popen:
    p = subprocess.Popen([f"{PATH}aptos-node", "--test", "--test-dir", TESTNET_DATA_DIR], stdout=subprocess.PIPE, preexec_fn=os.setsid)
    return p

def kill_testnet(pid):
    os.killpg(os.getpgid(pid), signal.SIGTERM)

def edit_testnet():
    with open(f"{TESTNET_DATA_DIR}/0/node.yaml", "w") as f:
        lines = f.read()
        try:
            config = yaml.safe_load(lines)
            print(config)
        except yaml.YAMLError as exc:
            print(exc)


def run_custom_testnet():
    p = run_testnet()
    pid = p.pid

    # wait a while for testnet to be live
    for _ in range(0,6): # wait for 60s
        time.sleep(10)
        try:
            with open(f"{TESTNET_DATA_DIR}/0/node.yaml", "r") as f:
                lines = f.read()
                if lines:
                    print("Testnet has valid config now")
                    print(lines)
                    break
                else:
                    print("Testnet does not have valid config yet. It's booting...")
        except:
            print("Testnet does not have config yet. It's probably not live")
            continue
        print("Waiting for testnet to be live...")

    kill_testnet(pid)
    edit_testnet()

if __name__ == "__main__":
    run_custom_testnet()
