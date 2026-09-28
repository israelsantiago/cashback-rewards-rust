# Comandos Cargo — suíte completa

Execute a partir da raiz do projeto.

## 1. Toolchain

```bash
rustc --version
cargo --version
rustup show active-toolchain
```

## 2. Formatação

```bash
cargo fmt --all
cargo fmt --all -- --check
```

## 3. Resolver/compilar tudo

```bash
cargo check --workspace --all-targets --all-features
cargo build --workspace --all-targets --all-features
cargo build --workspace --all-targets --all-features --release
```

## 4. Compilar a suíte sem executar

```bash
cargo test --workspace --all-targets --all-features --no-run
```

## 5. Suíte padrão

```bash
cargo test --workspace --all-targets
```

## 6. Suíte completa + PostgreSQL/Testcontainers

```bash
cargo test --workspace --all-targets --all-features -- --test-threads=1
```

## 7. Doc tests

```bash
cargo test --workspace --doc --all-features
```

## 8. Clippy

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## 9. Rustdoc

```bash
cargo doc --workspace --no-deps --all-features
```

## 10. Auditoria opcional

```bash
cargo audit
```

## 11. Gate completo

```bash
./scripts/verify-rust-suite.sh
```

A suíte completa requer Docker porque os testes PostgreSQL usam Testcontainers.
