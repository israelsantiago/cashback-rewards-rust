# Review

Before committing, inspect the changed code for:

- business-rule/spec compliance;
- domain independence from frameworks;
- HTTP handlers free of business logic;
- persistence concerns isolated in adapters;
- monetary calculations using `Decimal` with explicit rounding;
- tests covering boundary and counter-example behavior;
- accidental secrets, debug output or infrastructure leakage.
