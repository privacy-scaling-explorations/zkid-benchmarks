#!/bin/bash

echo "Setting up virtual environment at .venv with Python 3.9"

# Check if "uv" installed
if ! command -v uv >/dev/null; then
    echo "uv is not installed. Please install it and try again."
    exit 1
fi


# Create virtual environment
echo "Creating virtual environment..."
uv python install 3.9
uv venv --python 3.9
source .venv/bin/activate

# Update dependencies
echo "Installing dependencies"
python -m ensurepip --upgrade
python -m pip install cairo-lang || {
    echo "Failed to install cairo-lang."
    exit 1
}
