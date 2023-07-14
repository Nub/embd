#!/usr/bin/env bash

sudo -E env "PATH=$PATH" DEFMT_LOG=trace cargo rb minimal
