# ipak (Infinite Package)

**ipak** is a command-line tool written in Rust designed to manage packages, offering functionalities such as installation, removal, listing, and more. It serves as a lightweight package management system with support for metadata, dependency management, and project initialization.

## Installation

You can install `ipak` using Cargo, Rust's package manager:

```sh
cargo install --git https://github.com/The-Infinitys/ipm.ipak
```

Ensure you have Rust and Cargo installed on your system. Visit [rust-lang.org](https://www.rust-lang.org/tools/install) for installation instructions.

## Usage

`ipak` provides a variety of commands to manage packages and projects. Below are the primary commands and their usage:

### Package Management

- **Install a Package**
  ```sh
  ipak pkg install <package_file> [--local | --global]
  ```
  Installs a package from a specified file. Use `--local` for user-specific installation or `--global` for system-wide (requires superuser privileges).

- **List Installed Packages**
  ```sh
  ipak pkg list [--local | --global]
  ```
  Displays a list of installed packages. Defaults to local scope unless `--global` is specified or run as superuser.

- **Remove a Package**
  ```sh
  ipak pkg remove <package_name> [--local | --global]
  ```
  Uninstalls a package, keeping configuration files.

- **Purge a Package**
  ```sh
  ipak pkg purge <package_name> [--local | --global]
  ```
  Completely removes a package, including configuration files.

- **View Package Metadata**
  ```sh
  ipak pkg metadata <package_file>
  ```
  Displays metadata from a package archive.

### Project Management

- **Create a New Project**
  ```sh
  ipak project create --name <project_name> [--template <default|rust|python|dotnet|clang>]
  ```
  Initializes a new project with the specified name and template.

- **Initialize an Existing Project**
  ```sh
  ipak project init
  ```
  Adds `ipak` support to an existing project directory.

- **Build a Project**
  ```sh
  ipak project build [--release | --debug] [--shell <bash|zsh|csh|rbash>]
  ```
  Builds the project in the specified mode.

- **Package a Project**
  ```sh
  ipak project package [--target <source-build|normal|min>]
  ```
  Creates a package archive from the project.

### System Configuration

- **Configure ipak**
  ```sh
  ipak system configure [--local | --global]
  ```
  Sets up `ipak` configuration files in the specified scope.

### Help and Information

- **Display Help**
  ```sh
  ipak --help
  ```
  Shows available commands and options.

- **Display Version**
  ```sh
  ipak --version
  ```
  Prints the current version of `ipak`.

## Features

- **Package Metadata Management**: Uses `project.yaml` to define package metadata and dependencies.
- **Dependency Management**: Checks and resolves package dependencies during installation and removal.
- **Archive Support**: Creates and extracts package archives in various formats (e.g., `.tar.gz`, `.zip`).
- **Project Initialization**: Supports templates for Rust, Python, .NET, and C++ projects.

## Configuration File

The `project.yaml` file defines package metadata and dependencies. Here’s an example:

```yaml
about:
  author:
    name: "Author Name"
    email: "author@example.com"
  package:
    name: "example-package"
    version: "1.0.0"
    description: "An example package"
architecture:
  - "x86_64"
mode: "local"
relation:
  depend:
    - - "dep-package"
      - ">= 1.0, < 2.0"
  depend_cmds:
    - "git"
```

## Developer Information

### Project Structure

- `src/main.rs`: Entry point of the application.
- `src/modules/`: Core functionality modules (e.g., `pkg`, `project`, `system`).
- `src/utils/`: Utility modules (e.g., `archive`, `shell`, `color`).

### Contributing

Contributions are welcome! To contribute:

1. Fork the repository.
2. Create a feature branch (`git checkout -b feature-name`).
3. Commit your changes (`git commit -m "Add feature"`).
4. Push to the branch (`git push origin feature-name`).
5. Open a pull request.

Please ensure your code follows Rust conventions and includes appropriate tests.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details (Note: The actual license file is not included in the provided code; MIT is assumed based on common practice).
