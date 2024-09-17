# Diggity

Diggity is designed to provide similar functionality to Ruby's `dig` method. This module allows you to traverse nested structures to extract values using a specified path, or return a default value when the traversal is unsuccessful.

## Features

- Extract values from nested data structures using dot-separated paths.
- Return a default value or `None` if the path does not exist.
- Support for customizable path separators.

## Installation

```bash
pip install diggity
```

## Usage

```python
import diggity

data = {
    "user": {
        "name": "Alice",
        "age": 30,
        "preferences": {
            "languages": ["Python", "Rust", "Go"]
        }
    }
}

# Extracting a value using a dotted path
name = diggity.path(data, "user.name")  # Returns: "Alice"

# Extracting a non-existing value, returning None
hobby = diggity.path(data, "user.hobby")  # Returns: None

# Providing a default value for a non-existing path
hobby_with_default = diggity.path(data, "user.hobby", default="No hobby specified")  # Returns: "No hobby specified"

# Using a custom separator
favorite_language = diggity.path(data, "user:preferences:languages:0", sep=":")  # Returns: "Python"
```


## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.

## Acknowledgments

This project uses [PyO3](https://pyo3.rs/) to bridge Rust and Python. Special thanks to the contributors of the PyO3 library.


## TODOS

- currenlty it's x10 slower than native python
