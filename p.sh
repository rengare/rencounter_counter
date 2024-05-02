#!/bin/bash
cp target/release/rencounter_counter.exe ~/workspace/rencounter_bin/rencounter_counter_windows.exe
cp target/release/rencounter_counter ~/workspace/rencounter_bin/rencounter_counter_linux

zip -r rencounter_counter_linux_windows_macm1.zip ~/workspace/rencounter_bin
