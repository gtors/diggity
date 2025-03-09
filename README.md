# Diggity

**Diggity** is a Python library that provides functionality similar to Ruby's `dig` method.
It allows you to traverse nested data structures to extract values using a specified path or return a default value when the traversal is unsuccessful.

Additionally, it includes `coalesce` and `coalesce_logical` functions for handling optional values and finding the first non-`None` or truthy value in a sequence.

## Features
- **`dig_path`**: Extract value from nested data structures using dot-separated path.
- **`dig`**: Extract value from nested data structures using a sequence of keys, indices or attributes provided via `*args`.
- **`coalesce`**: Returns the first non-`None` value from a sequence of arguments.
- **`coalesce_logical`**: Returns the first truthy value from a sequence of arguments.

## Installation

To install **Diggity**, simply run the following command:

```bash
pip install diggity
```

## Usage

### Extracting Nested Values

You can extract values from nested data structures in various ways. Below are some examples.

```python
import diggity

data = {
    "users": [
        {
            "name": "Alice",
            "age": 30,
            "preferences": {
                "languages": ["Python", "Rust", "Go"]
            }
        },
    ]
}

# Extracting a value using a dotted path
name = diggity.dig_path(data, "users.0.name")  # Returns: "Alice"
# Or
name = diggity.dig(data, "users", 0, "name")  # Also returns: "Alice"

# Extracting a non-existing value, returning None
hobby = diggity.dig_path(data, "users.0.hobby")  # Returns: None
# Or
hobby = diggity.dig(data, "users", 0, "hobby")  # Also returns: None

# Providing a default value for a non-existing path
hobby_with_default = diggity.dig(data, "users", 0, "hobby", default="No hobby specified")  # Returns: "No hobby specified"

# Using a custom separator
favorite_language = diggity.dig_path(data, "users:0:preferences:languages:0", sep=":")  # Returns: "Python"
```

### Handling Optional Values with `coalesce`

The `coalesce` function returns the first non-`None` value from a sequence of arguments.

```python
import diggity

# Returns the first non-None value
result = diggity.coalesce(None, None, 42, None)  # Returns: 42

# Returns None if all values are None
result = diggity.coalesce(None, None, None)  # Returns: None

# Works with mixed types
result = diggity.coalesce(None, False, 0, "hello")  # Returns: False
```

### Finding the First Truthy Value with `coalesce_logical`

The `coalesce_logical` function returns the first truthy value from a sequence of arguments.

```python
import diggity

# Returns the first truthy value
result = diggity.coalesce_logical(None, False, 42, 0)  # Returns: 42

# Returns None if all values are falsy
result = diggity.coalesce_logical(None, False, 0, "")  # Returns: None

# Works with mixed types
result = diggity.coalesce_logical(None, False, "hello", 0)  # Returns: "hello"
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.

## Acknowledgments

This project uses [PyO3](https://pyo3.rs/) to bridge Rust and Python. Special thanks to the contributors of the PyO3 library.
