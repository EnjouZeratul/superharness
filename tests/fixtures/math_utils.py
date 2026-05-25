"""Math utilities for testing Agent feature-addition capabilities."""


def calculate_mean(numbers):
    """Calculate the arithmetic mean."""
    if not numbers:
        raise ValueError("Cannot calculate mean of empty list")
    return sum(numbers) / len(numbers)


def calculate_std(numbers):
    """Calculate standard deviation."""
    if not numbers:
        raise ValueError("Cannot calculate std of empty list")
    mean = calculate_mean(numbers)
    variance = sum((x - mean) ** 2 for x in numbers) / len(numbers)
    return variance ** 0.5


# Missing functions that Agent should add:
# - calculate_median()
# - calculate_mode()
# - calculate_percentile()