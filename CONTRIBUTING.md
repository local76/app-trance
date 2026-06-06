Contributing to rIdle

We are thrilled that you want to help improve rIdle! Contributions from the community are what make open-source projects so special. Please follow these guidelines to make sure your contribution matches the style and quality standards of the project.

Developer Environment Setup
To build and test rIdle locally:
  Make sure you have the standard Rust toolchain installed.
  Clone this repository.
  Check code formatting:
    cargo fmt --check
  Run standard compiler lints:
    cargo clippy
  Test the debug build:
    cargo run
  Build and package the final release with the custom resource compiler script:
    just build

Pull Request Process
  Fork the repository and create a new feature branch:
    git checkout -b feature/my-new-feature
  Write clean code and keep your changes focused.
  Make sure all compile checks and lints pass.
  Document any new features in the README.md or corresponding help manuals.
  Open a Pull Request detailing the purpose of your change and any design decisions you made.

TUI Design Principles
If you are modifying the user interface, please keep in mind:
  Aesthetics: We use high-contrast HSL/RGB tailored color themes. Do not use plain primaries (such as pure blue, pure red).
  Compact Core Grid: Keep core layouts wrapped so they support resizing without breaking borders.
