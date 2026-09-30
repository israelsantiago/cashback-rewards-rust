# Medição de Tempos da Suíte de Testes

## Instrumentação existente

O projeto utiliza `tracing` com o target:

```text
cashback_rewards_rust::test_timing
```

O `TestTimer` registra:

```text
test.started
test.completed
```

e informa `elapsed_ms`.

Na infraestrutura PostgreSQL também são registrados:

```text
postgres.starting
postgres.container_ready
postgres.pool_ready
postgres.migrations_completed
postgres.environment_ready
fixture.created
```

## Visualização imediata

Execute:

```bash
RUST_LOG='cashback_rewards_rust::test_timing=info' cargo test --workspace --all-targets --all-features --   --nocapture --test-threads=1
```

`--nocapture` é importante para visualizar a saída da instrumentação durante a execução dos testes.

## Salvar o trace

```bash
RUN_ID="$(date +%Y%m%d-%H%M%S)"
LOG_DIR="target/test-timings/$RUN_ID"
mkdir -p "$LOG_DIR"

RUST_LOG='cashback_rewards_rust::test_timing=info' cargo test --workspace --all-targets --all-features --   --nocapture --test-threads=1   2>&1 | tee "$LOG_DIR/full.log"
```

Arquivos esperados:

```text
target/test-timings/<timestamp>/
└── full.log
```

## Localizar os testes medidos

```bash
rg 'event="test.completed"' target/test-timings -g '*.log'
```

Para localizar a inicialização do PostgreSQL:

```bash
rg 'postgres\.(starting|container_ready|pool_ready|migrations_completed|environment_ready)'   target/test-timings
```

## Extrair apenas os tempos

```bash
rg 'event="test.completed"' target/test-timings -g '*.log'   | sed -E 's/.*test=([^ ]+).*elapsed_ms=([0-9]+).*/\1\t\2 ms/'   | sort -k2,2nr
```

Se a versão do formato do `tracing-subscriber` mudar e essa regex não casar, use primeiro:

```bash
rg 'event="test.completed"' target/test-timings -g '*.log'
```

e ajuste a extração à linha real gerada.

## Importante: tempo do teste x tempo do processo

Existem três métricas diferentes:

### 1. Tempo de compilação

Visto no `cargo test`/`cargo test --no-run`.

Não deve entrar no benchmark da suíte.

### 2. Tempo de infraestrutura

É o intervalo:

```text
postgres.starting
        ↓
postgres.environment_ready
```

Ele ocorre apenas quando o `OnceCell` inicializa o PostgreSQL daquele processo de teste.

### 3. Tempo do cenário

É o:

```text
test.started
        ↓
test.completed
```

Esse é o indicador principal para comparar cenários.

## Estado atual da cobertura da instrumentação

A instrumentação de `TestTimer` deve ser considerada parte do framework de observabilidade da suíte.

Na versão atualmente inspecionada, ela está explicitamente usada nos testes que exercitam PostgreSQL/acceptance e no suporte de fixtures. Para obter uma visão realmente uniforme de **cada função de teste**, o próximo refinamento é instanciar o timer também nos testes de:

- domain;
- application;
- adapter inbound/web.

Isso é desejável porque testes unitários podem ser tão rápidos que o custo agregado por camada fica invisível quando olhamos somente para o tempo do processo.

A instrumentação não deve alterar as regras de produção; ela pertence ao suporte de testes.

## Critério para decisões futuras

Não otimizar apenas o teste individual mais lento.

Primeiro observar:

```text
tempo total da suíte
        +
tempo por camada
        +
custo do PostgreSQL
        +
efeito do paralelismo
```

Uma alteração de isolamento só deve ser adotada quando demonstrar ganho mensurável sem degradar a confiabilidade ou tornar os testes difíceis de entender.

