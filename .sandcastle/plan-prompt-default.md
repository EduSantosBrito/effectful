# TASK

Analyze available implementation issues and identify work that can be safely done next.

# SOURCES

Use the issue body supplied by the runner as source of truth. The runner may supply:

- Local markdown issues from `issues/` or `.plans/` when present.
- Remote tracker issues already loaded by the runner when no local issue root exists.

Do not perform additional issue tracker queries unless the supplied issue body is insufficient to determine dependencies.

# DEPENDENCIES

Build a dependency graph. An issue is blocked by another issue if:

- It needs code, APIs, or decisions introduced by the other issue.
- It modifies overlapping modules where concurrent work likely creates conflicts.
- Its acceptance criteria depend on a prior vertical slice.

# OUTPUT

Output a JSON object wrapped in `<plan>` tags:

<plan>
{"issues":[{"file":"#123","title":"Example","branch":"sandcastle/issue-123-example"}]}
</plan>

Include only unblocked issues. If all are blocked, include the single safest next issue.

When complete, output {{COMPLETION_SIGNAL}}.
