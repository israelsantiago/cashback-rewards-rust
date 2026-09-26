# TDD cycle

Run one RED -> GREEN -> REFACTOR cycle.

- RED: make the smallest failing test describe the next behavior.
- GREEN: implement only enough behavior to pass.
- REFACTOR: remove duplication, improve names and preserve the hexagonal boundary.

Domain tests must not require Axum, SQLx or PostgreSQL.
