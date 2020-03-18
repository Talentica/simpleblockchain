#!/bin/bash
docker build -f build/Dockerfile --target simplebc_buildbase --tag simplebc_buildbase .
docker build -f build/Dockerfile --target simplebc_build --tag simplebc_build .
docker build -f build/Dockerfile --target simplebc --tag simplebc .
