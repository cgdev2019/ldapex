# Browsing and editing

Once connected the app shows a two-pane layout: a collapsible DIT tree
on the left and an attribute panel on the right. A tab row above the
tree swaps the tree for a filter-based search pane.

(todo: add screenshot of the main browser)

## Navigating the tree

- Click a triangle to expand or collapse a node. Children are fetched
  lazily with scope `onelevel`, so deep trees stay responsive.
- Click a row's label to select the entry — its attributes load in the
  right-hand panel.
- Entries use a minimal icon set based on their object classes:
  `organizationalUnit` (folder), `person`/`inetOrgPerson` (person),
  groups, and a generic dot for everything else.

## Reading attributes

The attribute panel lists every value from a base-scoped read. Binary
values are shown as `<binary — N b64>` to keep the UI usable; text
values have a *Copy* action on click. A search field on top filters
the visible attribute names.

## Creating an entry

1. Select the parent entry (or nothing, to create under the base DN).
2. Click **+ Entrée** / **+ Entry**, pick one or more object classes
   (abstract ones are hidden), fill the **MUST** attributes and the
   **MAY** attributes you care about.
3. Tweak the **RDN** (default `cn=New user`) and submit.

## Editing an entry

Click **Edit**. Each text attribute becomes a list of editable rows:
change a value in place, click `×` to drop a value, `+ value` to append
one, or add a brand new attribute by typing its name in the bottom
input. **Save** diffs the draft against the server copy and sends only
the minimum number of `add`/`replace`/`delete` operations. Binary
attributes stay read-only in this release.

## Searching

Switch the sidebar to **Search** and provide a base DN, an LDAP filter
(RFC 4515, e.g. `(uid=alice)`), an optional scope and a size limit.
Results are clickable and load into the attribute panel.

## Deleting an entry

With an entry selected, click **Delete** in the attribute panel and
confirm. Only leaf entries can be deleted — remove children first.

## Logs

The app writes daily-rotated logs to
`~/.ldapex/logs/ldapex.log.YYYY-MM-DD` plus stdout. The writer scrubs
`password=…` patterns before flushing, so the file is safe to attach
to a bug report.
