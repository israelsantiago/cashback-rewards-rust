# Estratégia de Testes — Cashback Rewards Rust

## Objetivo

Esta estratégia preserva as responsabilidades de teste observadas no projeto Java `section-13/solution`, mas adapta a execução às características de Rust, Axum e SQLx.

O objetivo é separar claramente:

- **regras de negócio**: rápidas e sem infraestrutura;
- **casos de uso**: rápidos, usando portas/dobles pequenos;
- **adapter inbound/web**: validação do contrato HTTP com as portas de entrada simuladas;
- **adapter outbound/persistence**: contrato real com PostgreSQL;
- **acceptance/end-to-end**: fluxo HTTP até PostgreSQL, usando a composição real da aplicação.

## Decisão arquitetural

A suíte utiliza a seguinte pirâmide:

```text
                    ACCEPTANCE / E2E
              HTTP → Axum → Services
                 → Pg*Repository
                    → PostgreSQL
                [poucos, mais lentos]

               PERSISTENCE INTEGRATION
              PgRepository → PostgreSQL
                 [mais numerosos]

                ADAPTER INBOUND/WEB
              HTTP → Controller → Port
                 [rápidos, sem BD]

                APPLICATION UNIT
             Service → Outbound Ports
               [rápidos, sem BD]

                    DOMAIN UNIT
             Models + Domain Services
                 [mais rápidos]
```

### Regras da decisão

1. **Domain não conhece infraestrutura.**
2. **Application depende de traits/ports, nunca de PostgreSQL concreto.**
3. **Controller tests simulam os inbound use cases** para verificar contrato HTTP sem transformar cada teste de controller em teste de integração.
4. **Persistence integration tests usam PostgreSQL real** em Testcontainers.
5. **Acceptance tests usam PostgreSQL real e a mesma composição de produção**, evitando uma composição de aplicação exclusiva para testes.
6. **Não usamos repositório em memória como substituto do teste de PostgreSQL.** Doubles existem apenas para testar regras de aplicação isoladamente.
7. **A base PostgreSQL é compartilhada por processo de teste**, por meio de `OnceCell`, mantendo o container vivo enquanto o processo estiver executando.
8. **Cada cenário usa uma `FixtureIdentity` com UUID** para criar nomes/MCC/customer IDs exclusivos. Isso evita `TRUNCATE` como mecanismo de isolamento normal.
9. Estado global real, atualmente o `default_cashback_rate`, recebe **lock explícito** (`default_rate_guard`) quando o teste o altera.
10. Agregações que atravessam clientes, como total por categoria, continuam sendo exercitadas contra PostgreSQL real.

## Por que não um container por teste

Um container por teste oferece isolamento máximo, porém transfere para cada teste:

```text
create container
→ wait PostgreSQL
→ connect
→ migrations
→ execute test
→ destroy container
```

Isso tende a elevar drasticamente o tempo da suíte e reduz a capacidade de paralelização eficiente.

A estratégia escolhida mantém esse custo uma vez por processo de teste:

```text
processo de teste
    ↓
1 PostgreSQL Testcontainer
    ↓
1 PgPool
    ↓
migrations uma vez
    ↓
N testes / N fixtures
```

## Por que usar fixture identity em vez de TRUNCATE

O isolamento normal é por identidade:

```text
Fixture A → merchant-x-AAAA
             customer-x-AAAA
             category-x-AAAA
             mcc-x-AAAA

Fixture B → merchant-x-BBBB
             customer-x-BBBB
             category-x-BBBB
             mcc-x-BBBB
```

Vantagens:

- evita reset de todas as tabelas a cada cenário;
- reduz custo de DDL/DML de limpeza;
- mantém os testes compatíveis com paralelização futura;
- torna agregações previsíveis porque cada cenário consulta seu próprio identificador.

Limite importante: **identity não isola estado global**. Por isso o default rate possui lock.

## Tempo de execução: princípio da medição

A medição deve separar:

### Cold start da infraestrutura

Inclui:

```text
Docker/Testcontainer
+ startup PostgreSQL
+ conexão do pool
+ migrations
```

Esses eventos são emitidos pelo namespace:

```text
cashback_rewards_rust::test_timing
```

com eventos como:

```text
postgres.starting
postgres.container_ready
postgres.pool_ready
postgres.migrations_completed
postgres.environment_ready
```

### Tempo do cenário

`TestTimer` mede:

```text
test.started
test.completed
```

com:

```text
kind
test
fixture
elapsed_ms
```

Assim não se deve comparar diretamente o primeiro teste de PostgreSQL com outro teste já executado depois que o container está pronto.

## Baseline de performance

Para obter uma linha de base reproduzível:

```bash
cargo test --no-run --workspace --all-targets --all-features

RUN_ID="$(date +%Y%m%d-%H%M%S)"
mkdir -p "target/test-timings/$RUN_ID"

RUST_LOG='cashback_rewards_rust::test_timing=info' cargo test   --workspace   --all-targets   --all-features   --   --nocapture   --test-threads=1   2>&1 | tee "target/test-timings/$RUN_ID/full.log"
```

O `--no-run` separa o custo de compilação do custo de execução.

`--test-threads=1` deve ser usado no benchmark-base porque reduz interferência entre testes. Depois disso, deve ser feita uma segunda medição com paralelismo normal.

## Medir por categoria

Para comparação entre camadas, a recomendação é executar os targets separadamente:

```bash
RUST_LOG='cashback_rewards_rust::test_timing=info' cargo test --test domain_tests -- --nocapture --test-threads=1
RUST_LOG='cashback_rewards_rust::test_timing=info' cargo test --test application_tests -- --nocapture --test-threads=1
RUST_LOG='cashback_rewards_rust::test_timing=info' cargo test --test adapter_in_web_tests -- --nocapture --test-threads=1
RUST_LOG='cashback_rewards_rust::test_timing=info' cargo test --features postgres-acceptance --test legacy_layout_tests -- --nocapture --test-threads=1
```

Os nomes exatos dos targets podem mudar quando os arquivos agregadores forem reorganizados; `cargo test -- --list` é a forma de conferir os testes/targets presentes no checkout.

## Como ler os tempos

Para uma primeira análise:

```text
Domain
  dezenas de ms ou menos → esperado

Application
  muito baixo → esperado

Web/controller
  baixo → esperado

Persistence
  maior → esperado, pois existe I/O PostgreSQL

Acceptance
  maior ainda → esperado, pois passa por HTTP + application + PostgreSQL
```

Não se deve usar esses valores como metas universais. O objetivo inicial é observar a distribuição e identificar regressões.

Para cada categoria, registrar pelo menos:

- quantidade de testes;
- tempo total;
- mediana;
- p95;
- teste mais lento;
- custo de inicialização do PostgreSQL;
- diferença entre execução serial e paralela.

## Repetibilidade

Para uma medição mais confiável, fazer:

```text
2–5 execuções de aquecimento
+
10 execuções medidas
```

e usar principalmente mediana e p95.

O primeiro processo de PostgreSQL deve ser reportado separadamente porque contém o custo de infraestrutura.

## Próxima etapa de otimização

A estratégia atual deve ser mantida enquanto os tempos forem aceitáveis.

Se a suíte crescer significativamente, comparar experimentalmente:

1. fixture identity no mesmo banco;
2. banco PostgreSQL por teste;
3. schema por teste;
4. `#[sqlx::test]`;
5. `cargo-nextest` para paralelização e isolamento por processo.

A decisão deve ser baseada em medição real do projeto, não somente em complexidade teórica.

## Resultado arquitetural esperado

O resultado desejado é:

```text
                    produção
                       │
             ┌─────────▼─────────┐
             │  composição real  │
             │  build_app(pool)  │
             └─────────┬─────────┘
                       │
          ┌────────────┴────────────┐
          ▼                         ▼
      controller                 service
          │                         │
          └────────────┬────────────┘
                       ▼
                 outbound ports
                       │
                       ▼
                 Pg repositories
                       │
                       ▼
                  PostgreSQL


 testes unitários ------------------------> sem PostgreSQL
 testes de persistence -------------------> PostgreSQL real
 acceptance ------------------------------> mesma composição + PostgreSQL real
```

