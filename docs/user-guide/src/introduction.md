# Introduction

Ldapex is a desktop application for browsing and editing an LDAP
directory. It is distributed as a single native executable per operating
system and is meant to replace web-based LDAP administrators for
day-to-day tasks: navigating the directory tree, reading and tweaking
attributes, creating or deleting entries, running search filters, and
keeping a list of saved connection profiles locally.

## Who is this guide for

- Directory administrators who need a light, offline-friendly browser
  for OpenLDAP, Active Directory, 389 Directory Server, and similar
  servers that speak LDAP v3.
- Developers who want to inspect or seed a test directory (the
  shipped integration fixture targets the bitnami OpenLDAP image).

## What you can do with Ldapex

- Connect to one or many directories over `ldap://`, `ldaps://` or
  StartTLS.
- Navigate the DIT as a collapsible tree.
- View an entry's attributes, copy values to the clipboard with a
  click, and filter the attribute list.
- Create, rename, modify and delete entries (simple RFC 4511 operations
  — no recursive delete).
- Run arbitrary search filters scoped to a base DN.
- Save connection profiles (URL, bind DN, base DN, optional password)
  in a portable TOML file under your home directory.

## What is explicitly out of scope

- SASL binds beyond simple authentication (planned).
- Schema editing or ACI management.
- Recursive or bulk operations that the UI cannot clearly guard.

## How the guide is organised

1. [Installation](./installation.md) — getting a working binary.
2. [First connection](./first-connection.md) — launching the app,
   saving a profile, and reaching the DIT.
3. [Browsing and editing](./browsing-and-editing.md) — day-to-day use:
   navigation, edits, search, and keyboard shortcuts.

(todo: add screenshot of the main window)
