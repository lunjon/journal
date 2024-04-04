# Journal

My go-to CLI tool for managing notes and to-dos.

Features:
- Simple interface for taking journals
- Workspaces: separate journals in different workspaces
  - For example work, home, etc.
- Exporting (ZIP)
- Templates
- Encryption

## Installation

Using cargo:

```sh
$ cargo install --locked --path .
```

## Workspaces

A _workspace_ is a group of related topics, e.g. "work".
Using workspaces allows you to separate journals in to different directories.

If not used, journals are put into the "default" workspace.

When exporting, these are respected as well.

## Templates
Use templates to create files with predefined content.

In order to define a template add the following to your configuration:

```toml
[template]
# The key is the file extension you define the template for.
# In this case it will define a template for markdown files.
md = """
---
created: {{DATE}}
---

# Title
"""
```

Note the following:
- The template is defined using a multiline string
- `{{DATE}}` is a _placeholder_ string that will be replaced with the current date
- Predefined placeholders:
  - `{{DATE}}`: the date when invoking the command

## Export

`jn` support basic export functionality using `jn export --target <target> [OPTIONS]`.

### Zip

Running `jn export --target zip` creates a zip-archive named `journals.{DATE}.zip`.

## Encryption

Journals can be encrypted by using a key. It uses symmetric encryption based on AES GCM.

