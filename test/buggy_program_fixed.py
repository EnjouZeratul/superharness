"""Simple buggy program for testing Continuum Agent SDK"""

def calculate_average(numbers):
    """Calculate average of a list of numbers"""
    if not numbers:
        return 0
    total = 0
    for n in numbers:
        total += n
    return total / len(numbers)


def find_max(items):
    """Find maximum value in a list"""
    max_val = items[0]
    for item in items:
        if item > max_val:
            max_val = item
    return max_val


def merge_lists(list1, list2):
    """Merge two lists"""
    result = []
    for item in list1:
        result.append(item)
    for item in list2:
        result.append(item)
    return result


def count_words(text):
    """Count words in text"""
    return len(text.split())


# Test cases
if __name__ == "__main__":
    # Test 1: Empty list
    try:
        avg = calculate_average([])
        print(f"Average of empty list: {avg}")
    except Exception as e:
        print(f"Error 1: {e}")

    # Test 2: Find max
    nums = [1, 5, 3, 9, 2]
    print(f"Max of {nums}: {find_max(nums)}")

    # Test 3: Merge lists
    merged = merge_lists([1, 2], [3, 4])
    print(f"Merged [1,2] + [3,4]: {merged}")

    # Test 4: Count words
    text = "hello world test"
    print(f"Words in '{text}': {count_words(text)}")