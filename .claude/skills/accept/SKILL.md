# Acceptance test

Turn exactly one reviewed specification rule into a failing executable acceptance test.

Prefer black-box HTTP tests through the Axum router. Keep setup data local to the test and never calculate the expected result by calling production business logic.

Stop after the test is red so the next step can implement it.
