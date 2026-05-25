"""Bug program for testing Agent bug-fixing capabilities."""


def find_max(items):
    """Find the maximum value in a list.

    Bug: Causes IndexError when list is empty.
    """
    max_val = items[0]  # Bug: IndexError on empty list
    for item in items[1:]:
        if item > max_val:
            max_val = item
    return max_val


def calculate_average(numbers):
    """Calculate the average of a list.

    Bug: Returns None for empty list instead of raising error.
    """
    if len(numbers) == 0:
        return None  # Should raise ValueError or return 0
    return sum(numbers) / len(numbers)


def divide_safely(a, b):
    """Divide two numbers safely.

    Bug: Doesn't handle division by zero.
    """
    return a / b  # Bug: ZeroDivisionError when b is 0


def concatenate_strings(parts):
    """Concatenate a list of strings.

    Bug: Doesn't handle None values.
    """
    result = ""
    for part in parts:
        result += part  # Bug: TypeError if part is None
    return result


def find_duplicates(items):
    """Find duplicate items in a list.

    Bug: Doesn't handle non-hashable items.
    """
    seen = set()
    duplicates = []
    for item in items:
        if item in seen:
            duplicates.append(item)
        seen.add(item)  # Bug: TypeError if item is unhashable (like dict)
    return duplicates
