#!/usr/bin/env bash

cd "$(dirname "$0")"

llms . "AGENTS.md,CLAUDE.md,GEMINI.md,LLXPRT.md,QWEN.md,WORK.md,external,vendor,vendor-local,build,external,issues,old-typf,target,*.ppm"
