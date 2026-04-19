# First connection

On first launch Ldapex opens the **Profiles** screen. You can either
connect once by filling the *Quick connection* form or save a reusable
profile for later.

(todo: add screenshot of the profile picker + login form)

## Quick connection

The bottom *Quick connection* form asks for the minimum a simple bind
needs:

- **URL**: `ldap://host:389`, `ldaps://host:636`, or `ldap://host:389`
  with StartTLS selected below.
- **Bind DN**: the DN you authenticate as, for example
  `cn=admin,dc=example,dc=org`. Leave empty for an anonymous bind.
- **Password**: only used in memory, never saved.
- **Base DN**: the root of the tree you want to browse,
  e.g. `dc=example,dc=org`.
- **TLS**: `None`, `StartTLS` or `LDAPS (ldaps://)`. The *None* option
  is meant for loopback or lab networks only.

Click **Connect**. On success the profile screen disappears and the
main browser opens.

## Saving a profile

1. Click **+ New** in the *Saved profiles* section.
2. Fill in the same fields as the quick form, plus a display name.
3. Optionally set a password:
   - leaving it empty means the app will ask at each connection;
   - filling it in saves the value (plain text) to
     `~/.ldapex/profiles.toml`. On Unix the file is created with
     `0600` permissions so only the current user can read it.

## Profile storage

Every profile — including any saved password — lives in a single
portable TOML file under your home directory:

```text
~/.ldapex/profiles.toml
```

You can copy that file to another machine to carry your profiles with
you. **Do not share it** if it contains passwords.

The *Export* button on the profile picker dumps the same payload as
JSON into your clipboard; *Import* reads a JSON blob back in. Both
operations include stored passwords.

## Switching language

The very right of the topbar holds a `EN / FR` selector. Your pick is
stored in `localStorage` under `ldapex.locale` so next launches come up
in the same language. Removing the key (or running Ldapex from a fresh
profile) falls back to the OS locale: French browsers default to
French, everything else defaults to English.
