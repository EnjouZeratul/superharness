# sh-python

Python bindings for Continuum.

## Installation

```bash
pip install sh-python
```

## Usage

```python
import sh_python

# Create an agent
agent = sh_python.Agent("my-agent")
agent.start()

# Create a session
session = agent.create_session()
session.add_user_message("Hello!")
```