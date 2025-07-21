#!/bin/bash

# Function to detect OS (reused from previous scripts)
detect_os() {
  OS_TYPE=$(uname -s)
  case "$OS_TYPE" in
    Linux*)     echo "linux" ;;
    Darwin*)    echo "macos" ;;
    CYGWIN*)    echo "windows" ;;
    MINGW*)     echo "windows" ;;
    MSYS*)      echo "windows" ;;
    *)          echo "unknown" ;;
  esac
}

# Function to get raw CPU details based on OS
get_cpu_details() {
  OS=$(detect_os)
  if [ "$OS" = "unknown" ]; then
    echo "Unsupported OS"
    return
  fi

  # Initialize defaults
  MODEL="unknown"
  VENDOR="unknown"
  FAMILY="unknown"
  MODEL_NUM="unknown"
  STEPPING="unknown"

  # Get raw CPU info
  if [ "$OS" = "linux" ]; then
    # Parse /proc/cpuinfo (take first CPU's info)
    CPU_INFO=$(cat /proc/cpuinfo)
    MODEL=$(echo "$CPU_INFO" | grep -m1 "model name" | cut -d':' -f2 | sed 's/^ *//')
    VENDOR=$(echo "$CPU_INFO" | grep -m1 "vendor_id" | cut -d':' -f2 | sed 's/^ *//')
    FAMILY=$(echo "$CPU_INFO" | grep -m1 "cpu family" | cut -d':' -f2 | sed 's/^ *//')
    MODEL_NUM=$(echo "$CPU_INFO" | grep -m1 "model" | cut -d':' -f2 | sed 's/^ *//')
    STEPPING=$(echo "$CPU_INFO" | grep -m1 "stepping" | cut -d':' -f2 | sed 's/^ *//')
  elif [ "$OS" = "macos" ]; then
    MODEL=$(sysctl -n machdep.cpu.brand_string)
    VENDOR=$(sysctl -n machdep.cpu.vendor 2>/dev/null || echo "unknown")
    FAMILY=$(sysctl -n machdep.cpu.family 2>/dev/null || echo "unknown")
    MODEL_NUM=$(sysctl -n machdep.cpu.model 2>/dev/null || echo "unknown")
    STEPPING=$(sysctl -n machdep.cpu.stepping 2>/dev/null || echo "unknown")
  elif [ "$OS" = "windows" ]; then
    # In Git Bash/MINGW/Cygwin, use wmic if available
    if command -v wmic >/dev/null 2>&1; then
      MODEL=$(wmic cpu get Name | sed -n '2p' | sed 's/\r//')
      VENDOR=$(wmic cpu get Manufacturer | sed -n '2p' | sed 's/\r//')
      FAMILY=$(wmic cpu get Family | sed -n '2p' | sed 's/\r//')
      MODEL_NUM=$(wmic cpu get Revision | sed -n '2p' | sed 's/\r//')  # Revision often combines model/stepping
      STEPPING=$(wmic cpu get Stepping | sed -n '2p' | sed 's/\r//')
    else
      MODEL="unknown (wmic not available)"
    fi
  fi

  # Replace spaces with underscores in all values
  MODEL=$(echo "${MODEL:-unknown}" | sed 's/ /_/g')
  VENDOR=$(echo "${VENDOR:-unknown}" | sed 's/ /_/g')
  FAMILY=$(echo "${FAMILY:-unknown}" | sed 's/ /_/g')
  MODEL_NUM=$(echo "${MODEL_NUM:-unknown}" | sed 's/ /_/g')
  STEPPING=$(echo "${STEPPING:-unknown}" | sed 's/ /_/g')

  # Output the results
  echo "CPU Model: $MODEL"
  echo "Vendor: $VENDOR"
  echo "Family: $FAMILY"
  echo "Model Number: $MODEL_NUM"
  echo "Stepping: $STEPPING"
}

# Run the function
get_cpu_details
