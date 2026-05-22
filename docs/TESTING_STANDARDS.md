# Testing Standards

## Coverage Requirements

| Module Type | Minimum Coverage | Target |
|-------------|------------------|--------|
| Core Business Logic | 80% | 90% |
| LLM Client / Config | 70% | 80% |
| Utilities / Helpers | 60% | 70% |

**Current Priority Modules:**

| Module | Current | Target | Priority |
|--------|---------|--------|----------|
| `tools/builtin.py` | 0% | 80% | P0 |
| `workflow/dag.py` | 0% | 80% | P0 |
| `llm/client.py` | 21% | 70% | P1 |
| `agent/intelligent.py` | 28% | 70% | P1 |
| `config/loader.py` | 48% | 80% | P1 |

---

## Forbidden Patterns

### ❌ DO NOT

```python
# 1. Empty test with only pass
def test_something(self):
    pass

# 2. Assert True/False without condition
def test_feature(self):
    assert True  # ❌ USELESS

# 3. No assertions at all
def test_calculation(self):
    result = calculate(1, 2)
    # No assertion! ❌

# 4. Vague assertion
def test_output(self):
    result = process()
    assert result is not None  # ❌ Too weak

# 5. Commented out test
# def test_old_feature(self):
#     assert True  # ❌ Remove or implement
```

### ✅ MUST DO

```python
# 1. Test actual behavior
def test_addition(self):
    result = add(2, 3)
    assert result == 5

# 2. Test edge cases
def test_empty_input(self):
    with pytest.raises(ValueError):
        process(None)

# 3. Test multiple scenarios
@pytest.mark.parametrize("input,expected", [
    ("hello", "HELLO"),
    ("World", "WORLD"),
    ("", ""),
])
def test_uppercase(self, input, expected):
    assert uppercase(input) == expected

# 4. Test error handling
def test_connection_timeout(self):
    client = Client(timeout=0.001)
    with pytest.raises(TimeoutError):
        client.connect()

# 5. Test integration
def test_full_workflow(self):
    agent = Agent()
    result = agent.run("task")
    assert result.status == "completed"
    assert result.steps > 0
```

---

## Test Naming Convention

```python
# Pattern: test_<function>_<scenario>_<expected>

def test_parse_config_valid_file_returns_dict(self):
    """Test parsing a valid config file returns a dictionary."""
    pass

def test_parse_config_missing_file_raises_error(self):
    """Test parsing missing file raises FileNotFoundError."""
    pass

def test_parse_config_invalid_json_raises_error(self):
    """Test parsing invalid JSON raises ValueError."""
    pass
```

---

## Test Structure (AAA Pattern)

```python
def test_feature(self):
    # ARRANGE - Set up test data
    config = {"key": "value"}
    loader = ConfigLoader()

    # ACT - Execute the function
    result = loader.load(config)

    # ASSERT - Verify the result
    assert result.key == "value"
    assert result.is_valid is True
```

---

## What Each Test Must Verify

| Test Type | Required Assertions |
|-----------|---------------------|
| Unit Test | Return value + type |
| Error Test | Exception type + message |
| Integration Test | End-to-end result + side effects |
| Mock Test | Mock called + correct args |

---

## Review Checklist

Before submitting tests, verify:

- [ ] Every `test_` function has at least one assertion
- [ ] No `assert True` or `assert False` without condition
- [ ] No `pass` statement in test body
- [ ] No commented-out tests
- [ ] Test name describes what is being tested
- [ ] Edge cases and error paths are tested
- [ ] Mock tests verify mock was called correctly

---

## Enforcement

**CI will fail if:**

1. Any test file has 0 assertions
2. Coverage drops below minimum threshold
3. `assert True` or `pass` detected in test body
4. Test function body is empty after docstring

**Violations will be rejected in PR review.**
