import pytest
import sys
from pathlib import Path

@pytest.fixture(autouse=True)
def add_project_to_path():
    """Add the project root to Python path."""
    project_root = Path(__file__).parent.parent
    sys.path.insert(0, str(project_root))
