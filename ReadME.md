# Grumpy Clippy

Grumpy Clippy is a Rust-based project designed to enhance code quality and provide helpful linting suggestions. This tool aims to assist developers in writing cleaner, more efficient, and idiomatic Rust code.

## Features

- Custom linting rules tailored for your project.
- Integration with Rust's Clippy for advanced code analysis.
- Easy-to-use interface for managing linting configurations.

## Installation

1. Clone the repository:
    ```bash
    git clone https://github.com/yourusername/grumpy_clippy.git
    ```
2. Navigate to the project directory:
    ```bash
    cd grumpy_clippy
    ```
3. Build the project:
    ```bash
    cargo build --release
    ```

## Usage

Grumpy Clippy provides several parameters to customize its behavior. These parameters can be set via the command-line interface (CLI) or through a configuration file (`.grumpyclippy.toml`).

## CLI Parameters

Below are the available CLI parameters:

- `--output-format <format>`  
  Specifies the output format. Supported formats: `txt`, `json`.

- `--max-complexity <number>`  
  Sets the maximum allowed complexity for functions. Default: `32`.

- `--grumpiness-level <level>`  
  Defines the level of grumpiness. Supported levels: `mild`, `rude`, `sarcastic`.

- `--watch-files <patterns>`  
  Specifies file patterns to watch for changes (e.g., `*.rs`, `*.md`).

- `--verbose`  
  Enables verbose output.

- `--rules-file <path>`  
  Path to a custom rules file (e.g., `rules.toml`).

- `--git-integration`  
  Enables Git-based analysis.

## Configuration File Parameters

You can also define parameters in a `.grumpyclippy.toml` file. Example:

```toml
grumpiness_level = "sarcastic"
verbose = false
watch_files = ["*.md"]
ignore_patterns = []
max_function_size = 50
max_complexity = 5
custom_rules = "custom.toml"
git_integration = false
rules_file = "rules.toml"
```

## Usage Examples

### Run with Default Settings
```bash
cargo run -- path/to/your/project
```

### Specify Parameters via CLI
```bash
cargo run -- --output-format json --max-complexity 10 --grumpiness-level rude
```

### Use a Configuration File
```bash
cargo run -- --config-file path/to/.grumpyclippy.toml
```

For more details, refer to the [src/cli.rs](src/cli.rs) and [src/config.rs](src/config.rs) files.

## Contributing

Contributions are welcome! Please follow these steps:
1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Submit a pull request with a detailed description of your changes.

## Contact

For questions or feedback, feel free to reach out via GitHub Issues.
