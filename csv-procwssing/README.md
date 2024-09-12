# CSV Data Processing in Rust

## Overview

This Rust project reads data from a CSV file, processes the data through a pipeline, and converts Celsius temperature readings to Fahrenheit. The program filters out records based on specific criteria and performs basic data transformations.

## Features

- Reads data from a CSV file.
- Parses each record into a structured format.
- Filters records based on specific criteria (e.g., location).
- Converts temperature readings from Celsius to Fahrenheit.
- Provides unit tests to ensure functionality.
- It simulates using data pipelines.

## Project Structure

- `src/main.rs`: Contains the main program logic, including reading the CSV file, processing the data, and printing the results.
- `tests/mod.rs`: Contains unit tests to verify the functionality of CSV reading and data processing.

## Requirements

- Rust :) only. It does not use any external library.

## Setup

1. **Clone the repository:**

   ```bash
   git clone https://github.com/janosvajda/rust-examples
   cd rust-examples/csv-procwssing/src
   cargo run
   cargo test
