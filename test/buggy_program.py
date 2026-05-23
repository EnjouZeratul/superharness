"""Simple buggy program for testing Continuum Agent SDK"""

def calculate_average(numbers):
    """Calculate average of a list of numbers"""
    # Bug: doesn't handle empty list
    total = 0
    for n in numbers:
        total += n
    return total / len(numbers)


def find_max(items):
    """Find maximum value in a list"""
    # Bug: wrong comparison logic
    max_val = items[0]
    for item in items:
        if item < max_val:  # Bug: should be > not <
            max_val = item
    return max_val


def merge_lists(list1, list2):
    """Merge two lists"""
    # Bug: doesn't actually merge, just returns list1
    result = []
    for item in list1:
        result.append(item)
    # Missing: list2 items not added!
    return result


def count_words(text):
    """Count words in text"""
    # Bug: counts spaces instead of words
    count = 0
    for char in text:
        if char == ' ':
            count += 1
    return count  # Should return count + 1 for actual word count


# Test cases that will fail due to bugs
if __name__ == "__main__":
    # Test 1: Empty list causes ZeroDivisionError
    try:
        avg = calculate_average([])
        print(f"Average: {avg}")
    except Exception as e:
        print(f"Error 1: {e}")

    # Test 2: Wrong max value
    nums = [1, 5, 3, 9, 2]
    print(f"Max (wrong): {find_max(nums)}")  # Returns 1 instead of 9

    # Test 3: Merge doesn't work
    merged = merge_lists([1, 2], [3, 4])
    print(f"Merged (wrong): {merged}")  # Returns [1, 2] instead of [1, 2, 3, 4]

    # Test 4: Word count wrong
    text = "hello world test"
    print(f"Words (wrong): {count_words(text)}")  # Returns 2 instead of 3