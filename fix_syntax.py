import os

def fix_file(filepath):
    if not os.path.exists(filepath):
        return

    with open(filepath, 'r') as f:
        content = f.read()

    content = content.replace("));.id();", ")).id();")

    with open(filepath, 'w') as f:
        f.write(content)

for file in [
    "engine/src/agent/systems.rs",
    "engine/src/persistence/io.rs",
    "engine/src/testing/determinism.rs"
]:
    fix_file(file)
